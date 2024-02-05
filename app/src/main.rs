#![feature(let_chains)]
use std::path::{Path, PathBuf};

use chrono::{DateTime, FixedOffset};
use clap::Parser;
use color_eyre::eyre::Result;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use tracing::info;
use up_ynab::*;

use crate::{
    cli::{Cli, Commands},
    config::Config,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    install_tracing();

    let cli = Cli::parse();
    let config = Figment::new()
        .merge(Toml::file(
            cli.config.unwrap_or(PathBuf::from("config.toml")),
        ))
        .extract::<Config>()?;

    use cli::{Balance, GetAccounts, GetTransactions};
    match cli.command {
        Commands::SyncTransactions {
            since,
            until,
            run_path,
            dry_run,
        } => sync(&config, run_path.as_deref(), since, until, Some(dry_run)).await?,
        Commands::GetAccounts(x) => match x {
            GetAccounts::Up => get_up_accounts(&config).await?,
            GetAccounts::Ynab => get_ynab_accounts(&config).await?,
        },
        Commands::GetTransactions(x) => match x {
            GetTransactions::Up { since, until } => {
                get_up_transactions(&config, since, until).await?
            }
            GetTransactions::Ynab { since } => get_ynab_transactions(&config, since).await?,
        },
        Commands::GetBudgets => get_ynab_budgets(&config).await?,
        Commands::Balance(x) => match x {
            Balance::Up {
                run_path,
                since,
                until,
            } => up_balance(&run_path, since, until)?,
            Balance::Ynab {
                run_path,
                since,
                until,
            } => ynab_balance(&run_path, since, until)?,
        },
    }

    Ok(())
}

async fn get_up_accounts(config: &Config) -> Result<()> {
    for account in up_ynab::fetch_up_accounts(config).await? {
        info!("{account:?}")
    }
    Ok(())
}

async fn get_up_transactions(
    config: &Config,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<()> {
    for transaction in up_ynab::fetch_up_transactions(config, since, until).await? {
        info!("{transaction:?}")
    }
    Ok(())
}

async fn get_ynab_accounts(config: &Config) -> Result<()> {
    for account in up_ynab::fetch_ynab_accounts(config).await? {
        info!("{account:?}")
    }
    Ok(())
}

async fn get_ynab_transactions(
    config: &Config,
    since: Option<DateTime<FixedOffset>>,
) -> Result<()> {
    for transaction in up_ynab::fetch_ynab_transactions(config, since).await? {
        info!("{transaction:?}")
    }
    Ok(())
}

async fn get_ynab_budgets(config: &Config) -> Result<()> {
    for budget in up_ynab::fetch_ynab_budgets(config).await? {
        info!("{budget:?}")
    }
    Ok(())
}

async fn sync(
    config: &Config,
    run_path: Option<&Path>,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
    dry_run: Option<bool>,
) -> Result<()> {
    up_ynab::sync(SyncArgs {
        config,
        run_path,
        since,
        until,
        dry_run,
    })
    .await
}

fn up_balance(
    run_path: &Path,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<()> {
    up_ynab::up_balance(run_path, since, until)
}

fn ynab_balance(
    run_path: &Path,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<()> {
    up_ynab::ynab_balance(run_path, since, until)
}

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let fmt_layer = fmt::layer();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("up_ynab=debug"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
