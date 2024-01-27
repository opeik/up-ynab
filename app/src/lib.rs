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

use chrono::Utc;
use color_eyre::eyre::{Context, ContextCompat, Error, Result};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, info};
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
        let entries = fs::read_dir(&path)
            .wrap_err(format!("failed to load run `{path_str}`",))?
            .map(|file_path| Self::read_entry::<T, _>(file_path?.path()))
            .collect::<Result<Vec<_>>>()
            .wrap_err(format!("failed to parse `{path_str}`"))?;
        Ok(entries)
    }
}
