#![feature(let_chains)]

pub mod balance;
pub mod cli;
pub mod config;
pub mod run;
pub mod transaction;
pub mod transfer;
pub mod up;
pub mod ynab;

use std::{collections::HashMap, path::Path};

use chrono::{DateTime, FixedOffset};
use color_eyre::eyre::{eyre, ContextCompat, Result};
use config::Config;
use futures::{StreamExt, TryStreamExt};
use itertools::Itertools;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
pub struct Account {
    pub name: String,
    pub up_id: String,
    pub ynab_id: Uuid,
    pub ynab_transfer_id: Uuid,
}

pub type UpTransaction = up_client::models::TransactionResource;
pub type UpAccount = up_client::models::AccountResource;
pub type YnabTransaction = ynab_client::models::TransactionDetail;
pub type NewYnabTransaction = ynab_client::models::SaveTransaction;
pub type YnabAccount = ynab_client::models::Account;
pub type YnabBudget = ynab_client::models::BudgetSummary;
pub use self::{transaction::Transaction, transfer::match_transfers};
use crate::{run::Run, transfer::TransferMatches};

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

pub async fn fetch_ynab_budgets(config: &Config) -> Result<Vec<YnabBudget>> {
    info!("fetching ynab budgets...");
    let ynab_client = ynab::Client::new(&config.ynab.api_token);
    ynab_client.budgets().send().await
}

pub struct SyncArgs<'a> {
    pub config: &'a Config,
    pub in_path: Option<&'a Path>,
    pub since: Option<DateTime<FixedOffset>>,
    pub until: Option<DateTime<FixedOffset>>,
    pub dry_run: Option<bool>,
}

pub fn normalize_up_transactions(
    up_transactions: &[UpTransaction],
    accounts: &[Account],
) -> Result<Vec<Transaction>> {
    let up_transactions = up_transactions
        .iter()
        .map(|x| Transaction::from_up(x.clone(), accounts))
        .collect::<Result<Vec<_>>>()?;
    let num_transactions = up_transactions.len();

    let (expenses, transfers) = up_transactions
        .into_iter()
        .partition::<Vec<_>, _>(|x| x.is_expense());

    let TransferMatches { matched, unmatched } = transfer::match_transfers(&transfers)?;
    if expenses.len() + transfers.len() != num_transactions
        || (matched.len() * 2) + unmatched.len() != transfers.len()
    {
        return Err(eyre!(
            "mismatched transaction count, this should never happen"
        ));
    }

    let (roundups, non_roundups) = unmatched
        .into_iter()
        .partition::<Vec<_>, _>(|x| x.is_round_up());

    let non_roundups = non_roundups
        .into_iter()
        .map(|x| {
            let kind = match &x.kind {
                transaction::Kind::Transfer { to, from } => {
                    warn!(
                        "converting unmatched transfer `{}` aka `{}` to an expense",
                        x.msg.as_deref().unwrap_or_default(),
                        x.id,
                    );
                    transaction::Kind::Expense {
                        to: to.to_owned(),
                        from_name: from.name.to_owned(),
                    }
                }
                x => x.to_owned(),
            };

            let mut new_transaction = x.clone();
            new_transaction.kind = kind;
            new_transaction
        })
        .collect::<Vec<_>>();

    if !non_roundups.is_empty() {
        let total = non_roundups
            .iter()
            .map(|x| x.amount)
            .reduce(|acc, e| acc + e)
            .unwrap_or_default();
        warn!("found sussy transactions totalling {total}:\n{non_roundups:#?}");
    }

    info!(
        "up: found {} expenses, {} transfers, {} roundups",
        expenses.len(),
        matched.len() * 2,
        roundups.len(),
    );

    let transactions = expenses
        .into_iter()
        .chain(roundups.into_iter().cloned())
        .chain(non_roundups)
        .chain(matched.into_iter().map(|pair| pair.from).cloned())
        .sorted_by(|a, b| Ord::cmp(&a.time, &b.time))
        .collect::<Vec<_>>();
    Ok(transactions)
}

fn missing_transactions<'a>(
    source_transactions: &'a [Transaction],
    remote_transactions: &'a [Transaction],
) -> Vec<&'a Transaction> {
    let source_transactions_by_id = source_transactions
        .iter()
        .map(|x| (x.id.as_str(), x))
        .collect::<HashMap<&str, &Transaction>>();

    let remote_transactions_by_id = remote_transactions
        .iter()
        .filter(|x| x.imported_id.is_some())
        .map(|x| (x.imported_id.as_deref().unwrap(), x))
        .collect::<HashMap<&str, &Transaction>>();

    let missing_transactions = source_transactions_by_id
        .keys()
        .map(|k| (k, remote_transactions_by_id.get(k)))
        .filter(|(_k, v)| v.is_none())
        .map(|(k, _v)| source_transactions_by_id.get(k).unwrap())
        .copied()
        .collect::<Vec<_>>();

    missing_transactions
}

pub async fn sync(args: SyncArgs<'_>) -> Result<()> {
    let ynab_client = ynab::Client::new(&args.config.ynab.api_token);

    let run = if let Some(in_path) = args.in_path {
        Run::read(in_path)?
    } else {
        run::fetch_run(args.config, args.since, args.until).await?
    };

    let budget_id = args
        .config
        .ynab
        .budget_id
        .as_ref()
        .map(|x| Uuid::parse_str(x))
        .wrap_err("missing budget id")??;
    let budget = run
        .ynab_budgets
        .wrap_err("missing ynab budgets")?
        .iter()
        .find(|x| x.id == budget_id)
        .cloned()
        .wrap_err("failed to find budget with id: `{budget_id}`")?;

    let accounts = match_accounts(
        &run.up_accounts.unwrap_or_default(),
        &run.ynab_accounts.unwrap_or_default(),
    )?;

    let ynab_transactions = run
        .ynab_transactions
        .map(|x| {
            x.into_iter()
                .map(|x| Transaction::from_ynab(x, &budget, &accounts))
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?;

    let up_transactions =
        normalize_up_transactions(&run.up_transactions.unwrap_or_default(), &accounts)?;

    let missing_transactions = ynab_transactions
        .as_ref()
        .map(|ynab_transactions| missing_transactions(&up_transactions, ynab_transactions))
        .unwrap_or_default();
    info!(
        "found {} missing up transactions on ynab",
        missing_transactions.len()
    );

    let new_ynab_transactions = missing_transactions
        .into_iter()
        .map(|x| x.clone().to_ynab())
        .inspect(|x| {
            if let Err(e) = x {
                error!("failed to convert to new ynab transaction: {e}")
            }
        })
        .collect::<Result<Vec<_>>>()?;

    if args.dry_run.unwrap_or_default() {
        info!("dry run, skipping creating ynab transactions");
        return Ok(());
    } else if new_ynab_transactions.is_empty() {
        info!("nothing to do, stopping...");
        return Ok(());
    }

    info!(
        "creating ynab {} transactions...",
        new_ynab_transactions.len()
    );
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
        error!("failed to create {num_missing} transactions");
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

pub fn up_balance(
    in_path: &Path,
    out_path: Option<&Path>,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<()> {
    let run = Run::read(in_path)?;
    let accounts = match_accounts(
        &run.up_accounts.unwrap_or_default(),
        &run.ynab_accounts.unwrap_or_default(),
    )?;

    let up_transactions =
        normalize_up_transactions(&run.up_transactions.unwrap_or_default(), &accounts)?;
    let balances = balance::running_balance(&up_transactions);

    for balance in &balances {
        if let Some(since) = since
            && balance.transaction.time <= since
        {
            continue;
        }

        if let Some(until) = until
            && balance.transaction.time >= until
        {
            continue;
        }

        info!("{balance}");
    }

    if let Some(out_path) = out_path {
        info!("writing balance CSV to `{}`", out_path.to_string_lossy());
        balance::write_balance_csv(&balances, out_path)?;
    }

    Ok(())
}

pub fn ynab_balance(
    in_path: &Path,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<()> {
    let run = Run::read(in_path)?;

    let accounts = match_accounts(
        &run.up_accounts.unwrap_or_default(),
        &run.ynab_accounts.unwrap_or_default(),
    )?;

    // TODO: this doesn't account for multiple budgets
    // TODO: fix double collect
    let budget = run
        .ynab_budgets
        .as_ref()
        .and_then(|x| x.first())
        .wrap_err("missing budget")?;
    let ynab_transactions = run
        .ynab_transactions
        .unwrap_or_default()
        .into_iter()
        .map(|x| Transaction::from_ynab(x, budget, &accounts))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sorted_by(|a, b| Ord::cmp(&a.time, &b.time))
        .collect::<Vec<_>>();

    let balances = balance::running_balance(&ynab_transactions);
    info!("calculated {} balances", balances.len());

    for balance in balances {
        if let Some(since) = since
            && balance.transaction.time <= since
        {
            continue;
        }

        if let Some(until) = until
            && balance.transaction.time >= until
        {
            continue;
        }

        let status = balance
            .values
            .iter()
            .map(|(k, v)| format!("  - {}: {}", k.name, v.amount))
            .join("\n  ");
        info!("{}: \n  {status}", balance.transaction.time)
    }

    Ok(())
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
                .find(|x| x.name.trim() == up_account_name.trim())
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
