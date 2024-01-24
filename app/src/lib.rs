pub mod cli;
pub mod config;
pub mod transaction;
pub mod up;
pub mod ynab;

use std::{fs::File, io::BufReader, path::Path};

use color_eyre::eyre::{Context, Result};
use tracing::debug;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
pub struct Account {
    pub name: String,
    pub up_id: String,
    pub ynab_id: Uuid,
    pub ynab_transfer_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct Run {
    pub up_transactions: Vec<UpTransaction>,
}

pub type Accounts = [Account];
pub type UpTransaction = up_client::models::TransactionResource;
pub type YnabTransaction = ynab_client::models::TransactionDetail;
pub type NewYnabTransaction = ynab_client::models::SaveTransaction;
pub use transaction::Transaction;

impl Run {
    pub fn write<P: AsRef<Path>>(run_path: P, run: &Run) -> Result<()> {
        for transaction in &run.up_transactions {
            Self::write_up_transaction(run_path.as_ref(), transaction)?;
        }

        debug!(
            "wrote {} up transactions to `{}`",
            run.up_transactions.len(),
            run_path.as_ref().to_string_lossy()
        );

        Ok(())
    }

    pub fn write_up_transaction<P: AsRef<Path>>(
        path: P,
        transaction: &UpTransaction,
    ) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy();
        let transactions_path = path.as_ref().join("up_transactions");
        std::fs::create_dir_all(&transactions_path)?;

        let file_path = transactions_path.join(format!(
            "{}-{}.json",
            transaction.attributes.created_at, transaction.id
        ));
        let file_path_str = &file_path.to_string_lossy();

        let file = File::create_new(&file_path)
            .wrap_err(format!("failed to create run directory `{path_str}`"))?;
        serde_json::to_writer_pretty(file, &transaction)
            .wrap_err(format!("failed to write up transaction to {file_path_str}"))?;
        Ok(())
    }

    pub fn read<P: AsRef<Path>>(path: P) -> Result<Run> {
        let path_str = path.as_ref().to_string_lossy();
        let up_transactions_path = path.as_ref().join("up_transactions");

        let up_transactions = std::fs::read_dir(up_transactions_path)
            .wrap_err(format!("failed to load run `{path_str}`",))?
            .map(|file_path| {
                let file = File::open(file_path?.path())?;
                let reader = BufReader::new(file);
                let up_transaction: UpTransaction = serde_json::from_reader(reader)?;
                Ok(up_transaction)
            })
            .collect::<Result<Vec<_>>>()
            .wrap_err(format!("failed to parse run `{path_str}`"))?;

        debug!(
            "read {} up transactions from `{path_str}`",
            up_transactions.len(),
        );

        Ok(Run { up_transactions })
    }
}
