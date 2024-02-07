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
    Sync(sync::Args),

    #[command(subcommand)]
    Get(get::Cmd),
}

pub mod sync {
    use super::*;

    /// Fetches a resource.
    #[derive(clap::Args)]
    pub struct Args {
        /// Only sync transaction since this date.
        #[arg(long)]
        pub since: Option<DateTime<FixedOffset>>,
        /// Only sync transaction until this date.
        #[arg(long)]
        pub until: Option<DateTime<FixedOffset>>,
        /// Previous run path.
        #[arg(long, value_name = "FILE")]
        pub in_path: Option<PathBuf>,
        /// Run command without making any changes.
        #[arg(long, default_value_t = false)]
        pub dry_run: bool,
    }
}

pub mod get {
    use super::*;

    /// Fetches a resource.
    #[derive(clap::Subcommand)]
    pub enum Cmd {
        #[command(subcommand)]
        Account(account::Cmd),
        #[command(subcommand)]
        Transaction(transaction::Cmd),
        #[command(subcommand)]
        Balance(balance::Cmd),
    }

    pub mod account {
        /// Fetches accounts.
        #[derive(clap::Subcommand)]
        pub enum Cmd {
            /// Fetches Up accounts.
            Up,
            /// Fetches Ynab accounts.
            Ynab,
        }
    }

    pub mod transaction {
        use super::*;

        /// Fetches transactions.
        #[derive(clap::Subcommand)]
        pub enum Cmd {
            Up(up::Args),
            Ynab(ynab::Args),
        }

        pub mod up {
            use super::*;

            /// Fetches Up accounts.
            #[derive(clap::Args)]
            pub struct Args {
                /// Only fetch transaction since this date.
                #[arg(long)]
                pub since: Option<DateTime<FixedOffset>>,
                /// Only fetch transaction until this date.
                #[arg(long)]
                pub until: Option<DateTime<FixedOffset>>,
            }
        }

        pub mod ynab {
            use super::*;

            /// Fetches Ynab accounts.
            #[derive(clap::Args)]
            pub struct Args {
                /// Only fetch transaction since this date.
                #[arg(long)]
                pub since: Option<DateTime<FixedOffset>>,
            }
        }
    }

    pub mod balance {
        use super::*;

        /// List running balance.
        #[derive(clap::Subcommand)]
        pub enum Cmd {
            Up(up::Args),
            Ynab(ynab::Args),
        }

        pub mod up {
            use super::*;

            /// List running Up balance.
            #[derive(clap::Args)]
            pub struct Args {
                /// Run input path.
                #[arg(long, value_name = "FILE")]
                pub in_path: Option<PathBuf>,
                /// CSV output path.
                #[arg(long, value_name = "FILE")]
                pub out_path: Option<PathBuf>,
                /// Only list balances since this date.
                #[arg(long)]
                pub since: Option<DateTime<FixedOffset>>,
                /// Only list balances until this date.
                #[arg(long)]
                pub until: Option<DateTime<FixedOffset>>,
            }
        }

        pub mod ynab {
            use super::*;

            /// List running Ynab balance.
            #[derive(clap::Args)]
            pub struct Args {
                /// Previous run path.
                #[arg(long, value_name = "FILE")]
                pub in_path: PathBuf,
                /// CSV output path.
                #[arg(long, value_name = "FILE")]
                pub out_path: Option<PathBuf>,
                /// Only list balances since this date.
                #[arg(long)]
                pub since: Option<DateTime<FixedOffset>>,
                /// Only list balances until this date.
                #[arg(long)]
                pub until: Option<DateTime<FixedOffset>>,
            }
        }
    }
}
