use std::path::PathBuf;

use chrono::{DateTime, FixedOffset};

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Config file path.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Previous run path.
    #[arg(long, value_name = "FILE")]
    pub run_path: Option<PathBuf>,

    /// Run command without making any changes.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Syncs transactions from Up to YNAB.
    SyncTransactions {
        #[arg(long)]
        since: Option<DateTime<FixedOffset>>,
        #[arg(long)]
        until: Option<DateTime<FixedOffset>>,
    },
    /// Fetches Up accounts.
    GetUpAccounts,
    /// Fetches Up transactions.
    GetUpTransactions {
        #[arg(long)]
        since: Option<DateTime<FixedOffset>>,
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
        since: Option<DateTime<FixedOffset>>,
    },
}
