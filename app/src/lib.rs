#![feature(let_chains)]

pub mod cli;
pub mod config;
pub mod transaction;
pub mod up;
pub mod ynab;

use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Duration, FixedOffset, Utc};
use color_eyre::eyre::{eyre, Context, ContextCompat, Result};
use config::Config;
use futures::{StreamExt, TryStreamExt};
use itertools::Itertools;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, error, info};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize)]
pub struct Account {
    pub name: String,
    pub up_id: String,
    pub ynab_id: Uuid,
    pub ynab_transfer_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct Run {
    pub path: PathBuf,
    pub up_transactions: Option<Vec<UpTransaction>>,
    pub up_accounts: Option<Vec<UpAccount>>,
    pub ynab_transactions: Option<Vec<YnabTransaction>>,
    pub ynab_accounts: Option<Vec<YnabAccount>>,
}

pub type UpTransaction = up_client::models::TransactionResource;
pub type UpAccount = up_client::models::AccountResource;
pub type YnabTransaction = ynab_client::models::TransactionDetail;
pub type NewYnabTransaction = ynab_client::models::SaveTransaction;
pub type YnabAccount = ynab_client::models::Account;
pub use transaction::Transaction;

pub async fn fetch_up_accounts(config: &Config) -> Result<Vec<UpAccount>> {
    info!("fetching up accounts...");
    let up_client = up::Client::new(&config.up.api_token);
    let accounts = up_client
        .accounts()
        .send()?
        .inspect_err(|e| error!("failed to fetch transaction: {e}"))
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    Ok(accounts)
}

pub async fn fetch_up_transactions(
    config: &Config,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<Vec<UpTransaction>> {
    info!("fetching up transactions...");
    let up_client = up::Client::new(&config.up.api_token);
    let transactions = up_client
        .transactions()
        .filter_since(since)
        .filter_until(until)
        .send()?
        .inspect_err(|e| error!("failed to fetch transaction: {e}"))
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    Ok(transactions)
}

pub async fn fetch_ynab_transactions(
    config: &Config,
    since: Option<DateTime<FixedOffset>>,
) -> Result<Vec<YnabTransaction>> {
    info!("fetching ynab transactions...");
    let ynab_client = ynab::Client::new(&config.ynab.api_token);
    let budget_id = config
        .ynab
        .budget_id
        .as_ref()
        .wrap_err("missing budget id")?;
    let transactions = ynab_client
        .transactions()
        .budget_id(budget_id)
        .since_date(since)
        .send()
        .await?;
    Ok(transactions)
}

pub async fn fetch_ynab_accounts(config: &Config) -> Result<Vec<YnabAccount>> {
    info!("fetching ynab accounts...");
    let ynab_client = ynab::Client::new(&config.ynab.api_token);
    let budget_id = config
        .ynab
        .budget_id
        .as_ref()
        .wrap_err("missing budget id")?;
    let accounts = ynab_client.accounts().budget_id(budget_id).send().await?;
    Ok(accounts)
}

pub async fn fetch_run(
    config: &Config,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<Run> {
    let mut run = Run::new();

    let up_accounts = fetch_up_accounts(config).await?;
    run.write_up_accounts(&up_accounts)?;

    let ynab_accounts = fetch_ynab_accounts(config).await?;
    run.write_ynab_accounts(&ynab_accounts)?;

    let up_transactions = fetch_up_transactions(config, since, until).await?;
    run.write_up_transactions(&up_transactions)?;

    let ynab_transactions = fetch_ynab_transactions(config, since).await?;
    run.write_ynab_transactions(&ynab_transactions)?;

    run.up_accounts = Some(up_accounts);
    run.up_transactions = Some(up_transactions);
    run.ynab_accounts = Some(ynab_accounts);
    run.ynab_transactions = Some(ynab_transactions);
    Ok(run)
}

#[derive(Debug, Clone)]
pub struct TransferPair<'a> {
    pub to: &'a Transaction,
    pub from: &'a Transaction,
}

pub struct SyncArgs<'a> {
    pub config: &'a Config,
    pub run_path: Option<&'a Path>,
    pub since: Option<DateTime<FixedOffset>>,
    pub until: Option<DateTime<FixedOffset>>,
    pub dry_run: Option<bool>,
}

pub async fn sync(args: SyncArgs<'_>) -> Result<()> {
    let ynab_client = ynab::Client::new(&args.config.ynab.api_token);
    let budget_id = args
        .config
        .ynab
        .budget_id
        .as_ref()
        .wrap_err("missing budget id")?;

    let run = if let Some(run_path) = args.run_path {
        Run::read(run_path)?
    } else {
        fetch_run(args.config, args.since, args.until).await?
    };

    let accounts = match_accounts(
        &run.up_accounts.unwrap_or_default(),
        &run.ynab_accounts.unwrap_or_default(),
    )?;

    let up_transactions = run
        .up_transactions
        .unwrap_or_default()
        .into_iter()
        .flat_map(|x| Transaction::from_up(x, &accounts))
        .collect::<Vec<_>>();

    let (expenses, transfers) =
        up_transactions
            .into_iter()
            .partition::<Vec<_>, _>(|x| match &x.kind {
                transaction::Kind::Expense {
                    to: _,
                    from_name: _,
                } => true,
                transaction::Kind::Transfer { to: _, from: _ } => false,
            });

    let (matched, unmatched) = match_transfers(&transfers)?;
    let (roundups, non_roundups) = unmatched.iter().partition::<Vec<&Transaction>, _>(|x| {
        x.msg.as_ref().map(|x| x == "Round Up").unwrap_or_default()
    });

    if !non_roundups.is_empty() {
        return Err(eyre!("found sussy transactions: {non_roundups:?}"));
    }

    info!(
        "found {} expenses, {} transfers, {} roundups",
        expenses.len(),
        matched.len() * 2,
        roundups.len(),
    );

    // let test = run
    //     .ynab_transactions
    //     .unwrap_or_default()
    //     .into_iter()
    //     .map(|x| Transaction::from_ynab(x, &accounts))
    //     .inspect(|x| {
    //         if let Err(e) = x {
    //             error!("failed to convert ynab transaction: {e}");
    //         }
    //     })
    //     .flatten()
    //     .collect::<Vec<_>>();

    let transactions = expenses
        .iter()
        .chain(roundups.into_iter())
        .chain(non_roundups.into_iter())
        .chain(matched.iter().map(|pair| pair.from))
        .collect::<Vec<_>>();

    let new_ynab_transactions = transactions
        .iter()
        .map(|x| x.to_ynab())
        .inspect(|x| {
            if let Err(e) = x {
                error!("failed to convert to new ynab transaction: {e}")
            }
        })
        .collect::<Result<Vec<_>>>()?;

    if args.dry_run.unwrap_or_default() {
        info!("dry run, skipping creating ynab transactions");
        return Ok(());
    }

    info!("creating ynab transactions...");
    let num_transactions = new_ynab_transactions.len();
    let response = ynab_client
        .new_transactions()
        .budget_id(budget_id)
        .transactions(new_ynab_transactions)
        .send()
        .await?;

    let num_missing =
        num_transactions - response.transactions.as_ref().unwrap_or(&Vec::new()).len();

    if num_missing != 0 {
        return Err(eyre!("failed to create {num_missing} transactions"));
    }

    if let Some(duplicate_ids) = response.duplicate_import_ids
        && !duplicate_ids.is_empty()
    {
        return Err(eyre!(
            "found duplicate transaction ids: {}",
            duplicate_ids.iter().join(", ")
        ));
    }

    Ok(())
}

fn match_transfers(transfers: &[Transaction]) -> Result<(Vec<TransferPair>, Vec<&Transaction>)> {
    let mut matched = Vec::new();
    let mut unmatched = Vec::new();

    let groups = transfers
        .iter()
        .into_group_map_by(|x| x.amount.amount.abs());

    // TODO: make less gross
    for group in &groups {
        let (mut tos, mut froms) = group
            .1
            .iter()
            .map(|x| Some(*x))
            .partition::<Vec<Option<&Transaction>>, _>(|x| {
                x.unwrap().amount.amount.is_sign_negative()
            });

        let mut pairs = Vec::new();
        for to in &mut tos {
            for from in &mut froms {
                if let Some(to_inner) = to
                    && let Some(from_inner) = from
                {
                    let d = (from_inner.time - to_inner.time).abs();
                    if d <= Duration::seconds(15) {
                        pairs.push(TransferPair {
                            to: to_inner,
                            from: from_inner,
                        });
                        *to = None;
                        *from = None;
                    }
                }
            }
        }

        let mut remainder = tos
            .into_iter()
            .chain(froms.into_iter())
            .flatten()
            .collect::<Vec<_>>();

        matched.append(&mut pairs);
        unmatched.append(&mut remainder);
    }

    Ok((matched, unmatched))
}

pub fn match_accounts(
    up_accounts: &[UpAccount],
    ynab_accounts: &[YnabAccount],
) -> Result<Vec<Account>> {
    let accounts = up_accounts
        .iter()
        .map(|up_account| {
            let up_account_name = up_account.attributes.display_name.clone();
            let ynab_account = ynab_accounts
                .iter()
                .find(|x| x.name == up_account_name)
                .wrap_err(format!(
                    "failed to match up account `{up_account_name}` to ynab account"
                ))?;
            Ok(Account {
                name: up_account_name,
                up_id: up_account.id.clone(),
                ynab_id: ynab_account.id,
                ynab_transfer_id: ynab_account
                    .transfer_payee_id
                    .wrap_err("missing ynab transfer id")?,
            })
        })
        .inspect(|x: &Result<Account>| {
            if let Err(e) = x {
                error!("{e}");
            };
        })
        .flatten()
        .collect::<Vec<Account>>();
    info!("matched {} accounts", accounts.len());
    Ok(accounts)
}

impl Default for Run {
    fn default() -> Self {
        let date = Utc::now().to_rfc3339();
        let path = PathBuf::from(format!("runs/{date}"));
        info!("starting new run at `{}`", path.to_string_lossy());

        Self {
            path,
            up_transactions: None,
            up_accounts: None,
            ynab_transactions: None,
            ynab_accounts: None,
        }
    }
}

impl Run {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn write(&self) -> Result<()> {
        self.up_accounts
            .as_ref()
            .map(|x| Self::write_up_accounts(self, x))
            .transpose()?;

        self.up_transactions
            .as_ref()
            .map(|x| Self::write_up_transactions(self, x))
            .transpose()?;

        self.ynab_accounts
            .as_ref()
            .map(|x| Self::write_ynab_accounts(self, x))
            .transpose()?;

        self.ynab_transactions
            .as_ref()
            .map(|x| Self::write_ynab_transactions(self, x))
            .transpose()?;

        Ok(())
    }

    pub fn write_up_transactions(&self, transactions: &[UpTransaction]) -> Result<()> {
        let path = self.path.join("up_transactions");
        Self::write_entries::<UpTransaction, _, _>(&path, transactions, |x| {
            PathBuf::from(&format!("{}-{}.json", x.attributes.created_at, x.id))
        })?;
        debug!("wrote up transactions to {}", path.to_string_lossy());
        Ok(())
    }

    pub fn write_up_accounts(&self, accounts: &[UpAccount]) -> Result<()> {
        let path = self.path.join("up_accounts");
        Self::write_entries::<UpAccount, _, _>(&path, accounts, |x| {
            PathBuf::from(&format!("{}.json", x.id))
        })?;
        debug!("wrote up accounts to {}", path.to_string_lossy());
        Ok(())
    }

    pub fn write_ynab_accounts(&self, accounts: &[YnabAccount]) -> Result<()> {
        let path = self.path.join("ynab_accounts");
        Self::write_entries::<YnabAccount, _, _>(&path, accounts, |x| {
            PathBuf::from(&format!("{}.json", x.id))
        })?;
        debug!("wrote ynab accounts to {}", path.to_string_lossy());
        Ok(())
    }

    pub fn write_ynab_transactions(&self, transactions: &[YnabTransaction]) -> Result<()> {
        let path = self.path.join("ynab_transactions");
        Self::write_entries::<YnabTransaction, _, _>(
            self.path.join("ynab_transactions"),
            transactions,
            |x| PathBuf::from(&format!("{}-{}.json", x.date, x.id)),
        )?;
        debug!("wrote ynab transactions to {}", path.to_string_lossy());
        Ok(())
    }

    fn read_up_transactions<P: AsRef<Path>>(path: P) -> Result<Vec<UpTransaction>> {
        Self::read_entries::<UpTransaction, _>(path.as_ref().join("up_transactions"))
    }

    fn read_up_accounts<P: AsRef<Path>>(path: P) -> Result<Vec<UpAccount>> {
        Self::read_entries::<UpAccount, _>(path.as_ref().join("up_accounts"))
    }

    fn read_ynab_transactions<P: AsRef<Path>>(path: P) -> Result<Vec<YnabTransaction>> {
        Self::read_entries::<YnabTransaction, _>(path.as_ref().join("ynab_transactions"))
    }

    fn read_ynab_accounts<P: AsRef<Path>>(path: P) -> Result<Vec<YnabAccount>> {
        Self::read_entries::<YnabAccount, _>(path.as_ref().join("ynab_accounts"))
    }

    pub fn read<P: AsRef<Path>>(path: P) -> Result<Run> {
        info!("opening run: `{}", path.as_ref().to_string_lossy());
        Ok(Run {
            path: path.as_ref().to_path_buf(),
            up_transactions: Some(Self::read_up_transactions(path.as_ref())?),
            up_accounts: Some(Self::read_up_accounts(path.as_ref())?),
            ynab_transactions: Some(Self::read_ynab_transactions(path.as_ref())?),
            ynab_accounts: Some(Self::read_ynab_accounts(path.as_ref())?),
        })
    }

    fn write_entry<T: Serialize, P: AsRef<Path>>(path: P, entry: &T) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy();
        fs::create_dir_all(path.as_ref().parent().wrap_err("unable to get parent")?)?;
        let file = File::create_new(&path)
            .wrap_err(format!("failed to create directory for `{path_str}`"))?;
        serde_json::to_writer_pretty(file, &entry)
            .wrap_err(format!("failed to write entry to {path_str}"))?;
        Ok(())
    }

    fn write_entries<T: Serialize, P: AsRef<Path>, F: Fn(&T) -> PathBuf>(
        path: P,
        entries: &[T],
        f: F,
    ) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy();
        entries
            .iter()
            .try_for_each(|entry| Self::write_entry::<T, _>(path.as_ref().join(f(entry)), entry))
            .wrap_err(format!("failed to write `{path_str}`"))?;
        Ok(())
    }

    fn read_entry<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T> {
        let path_str = path.as_ref().to_string_lossy();
        let entry = File::open(&path)
            .map(BufReader::new)
            .map(serde_json::from_reader::<_, T>)
            .wrap_err(format!("failed to parse `{path_str}`"))??;
        Ok(entry)
    }

    fn read_entries<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<Vec<T>> {
        let path_str = path.as_ref().to_string_lossy();
        if !path.as_ref().exists() {
            error!(
                "run component `{}` missing, skipping...",
                path.as_ref()
                    .file_name()
                    .wrap_err("missing dir")?
                    .to_string_lossy()
            );
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&path)
            .wrap_err(format!("failed to load run `{path_str}`",))?
            .map(|file_path| Self::read_entry::<T, _>(file_path?.path()))
            .collect::<Result<Vec<_>>>()
            .wrap_err(format!("failed to parse `{path_str}`"))?;
        Ok(entries)
    }
}
