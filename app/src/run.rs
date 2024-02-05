use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

use chrono::{DateTime, FixedOffset, Utc};
use color_eyre::eyre::{eyre, Context, ContextCompat, Result};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, error, info};

use crate::{config::Config, UpAccount, UpTransaction, YnabAccount, YnabBudget, YnabTransaction};

#[derive(Clone, Debug)]
pub struct Run {
    pub path: PathBuf,
    pub up_transactions: Option<Vec<UpTransaction>>,
    pub up_accounts: Option<Vec<UpAccount>>,
    pub ynab_transactions: Option<Vec<YnabTransaction>>,
    pub ynab_accounts: Option<Vec<YnabAccount>>,
    pub ynab_budgets: Option<Vec<YnabBudget>>,
}

pub async fn fetch_run(
    config: &Config,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<Run> {
    let mut run = Run::new();

    let up_accounts = crate::fetch_up_accounts(config).await?;
    run.write_up_accounts(&up_accounts)?;

    let ynab_accounts = crate::fetch_ynab_accounts(config).await?;
    run.write_ynab_accounts(&ynab_accounts)?;

    let ynab_budgets = crate::fetch_ynab_budgets(config).await?;
    run.write_ynab_budgets(&ynab_budgets)?;

    let up_transactions = crate::fetch_up_transactions(config, since, until).await?;
    run.write_up_transactions(&up_transactions)?;

    let ynab_transactions = crate::fetch_ynab_transactions(config, since).await?;
    run.write_ynab_transactions(&ynab_transactions)?;

    run.up_accounts = Some(up_accounts);
    run.up_transactions = Some(up_transactions);
    run.ynab_accounts = Some(ynab_accounts);
    run.ynab_transactions = Some(ynab_transactions);
    run.ynab_budgets = Some(ynab_budgets);
    Ok(run)
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
            ynab_budgets: None,
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
        Self::write_entries::<YnabTransaction, _, _>(&path, transactions, |x| {
            PathBuf::from(&format!("{}-{}.json", x.date, x.id))
        })?;
        debug!("wrote ynab transactions to {}", path.to_string_lossy());
        Ok(())
    }

    pub fn write_ynab_budgets(&self, budgets: &[YnabBudget]) -> Result<()> {
        let path = self.path.join("ynab_budgets");
        Self::write_entries::<YnabBudget, _, _>(&path, budgets, |x| {
            PathBuf::from(&format!("{}.json", x.id))
        })?;
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

    fn read_ynab_budgets<P: AsRef<Path>>(path: P) -> Result<Vec<YnabBudget>> {
        Self::read_entries::<YnabBudget, _>(path.as_ref().join("ynab_budgets"))
    }

    pub fn read<P: AsRef<Path>>(path: P) -> Result<Run> {
        info!("opening run: `{}", path.as_ref().to_string_lossy());
        if !path.as_ref().exists() {
            return Err(eyre!(
                "run missing at path `{}`",
                path.as_ref().to_string_lossy()
            ));
        }

        Ok(Run {
            path: path.as_ref().to_path_buf(),
            up_transactions: Some(Self::read_up_transactions(path.as_ref())?),
            up_accounts: Some(Self::read_up_accounts(path.as_ref())?),
            ynab_transactions: Some(Self::read_ynab_transactions(path.as_ref())?),
            ynab_accounts: Some(Self::read_ynab_accounts(path.as_ref())?),
            ynab_budgets: Some(Self::read_ynab_budgets(path.as_ref())?),
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
