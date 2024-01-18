use std::path::PathBuf;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Config file path.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand)]
#[command(rename_all = "snake_case")]
pub enum Commands {
    /// Fetches all Up accounts.
    GetUpAccounts,
    /// Fetches all Up transactions.
    GetUpTransactions,
    /// Fetches all YNAB accounts.
    GetYnabAccounts,
    /// Fetches all YNAB budgets.
    GetYnabBudgets,
    /// Fetches all YNAB transactions.
    GetYnabTransactions,
    /// Syncs all transactions from Up to YNAB.
    Sync,
}
