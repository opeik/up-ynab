use std::path::PathBuf;

use chrono::{DateTime, FixedOffset};

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
    /// Syncs transactions from Up to YNAB.
    SyncTransactions {
        /// Only sync transaction since this date.
        #[arg(long)]
        since: Option<DateTime<FixedOffset>>,
        /// Only sync transaction until this date.
        #[arg(long)]
        until: Option<DateTime<FixedOffset>>,
        /// Previous run path.
        #[arg(long, value_name = "FILE")]
        in_path: Option<PathBuf>,
        /// Run command without making any changes.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Fetches accounts.
    #[command(subcommand)]
    GetAccounts(GetAccounts),
    /// Fetches transactions.
    #[command(subcommand)]
    GetTransactions(GetTransactions),
    /// Fetches YNAB budgets.
    GetBudgets,
    /// List running balance for a past run.
    #[command(subcommand)]
    Balance(Balance),
}

#[derive(clap::Subcommand)]
pub enum GetAccounts {
    /// Fetches Up accounts.
    Up,
    /// Fetches Ynab accounts.
    Ynab,
}

#[derive(clap::Subcommand)]
pub enum GetTransactions {
    /// Fetches Up accounts.
    Up {
        /// Only fetch transaction since this date.
        #[arg(long)]
        since: Option<DateTime<FixedOffset>>,
        /// Only fetch transaction until this date.
        #[arg(long)]
        until: Option<DateTime<FixedOffset>>,
    },
    /// Fetches Ynab accounts.
    Ynab {
        /// Only fetch transaction since this date.
        #[arg(long)]
        since: Option<DateTime<FixedOffset>>,
    },
}

#[derive(clap::Subcommand)]

pub enum Balance {
    /// List running Up balance.
    Up {
        /// Run input path.
        #[arg(long, value_name = "FILE")]
        in_path: PathBuf,
        /// CSV output path.
        #[arg(long, value_name = "FILE")]
        out_path: Option<PathBuf>,
        /// Only list balances since this date.
        #[arg(long)]
        since: Option<DateTime<FixedOffset>>,
        /// Only list balances until this date.
        #[arg(long)]
        until: Option<DateTime<FixedOffset>>,
    },
    /// List running Ynab balance.
    Ynab {
        /// Previous run path.
        #[arg(long, value_name = "FILE")]
        in_path: PathBuf,
        /// Only list balances since this date.
        #[arg(long)]
        since: Option<DateTime<FixedOffset>>,
        /// Only list balances until this date.
        #[arg(long)]
        until: Option<DateTime<FixedOffset>>,
    },
}
