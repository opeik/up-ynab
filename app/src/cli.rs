use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, Utc};

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Config file path.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Fetches Up accounts.
    GetUpAccounts,
    /// Fetches Up transactions.
    GetUpTransactions {
        #[arg(long)]
        from: Option<DateTime<FixedOffset>>,
        #[arg(long)]
        until: Option<DateTime<FixedOffset>>,
    },
    /// Fetches YNAB accounts.
    GetYnabAccounts,
    /// Fetches YNAB budgets.
    GetYnabBudgets,
    /// Fetches YNAB transactions.
    GetYnabTransactions {
        #[arg(long)]
        from: Option<DateTime<FixedOffset>>,
    },
    /// Syncs transactions from Up to YNAB.
    Sync {
        #[arg(long)]
        from: Option<DateTime<FixedOffset>>,
        #[arg(long)]
        until: Option<DateTime<FixedOffset>>,
    },
    /// Configures up-ynab.
    Setup,
}
