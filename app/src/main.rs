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

    use cli::{Accounts, Balance, Transactions};
    match cli.command {
        Commands::Sync {
            since,
            until,
            in_path,
            dry_run,
        } => sync(&config, in_path.as_deref(), since, until, Some(dry_run)).await?,
        Commands::Accounts(x) => match x {
            Accounts::Up => get_up_accounts(&config).await?,
            Accounts::Ynab => get_ynab_accounts(&config).await?,
        },
        Commands::Transactions(x) => match x {
            Transactions::Up { since, until } => get_up_transactions(&config, since, until).await?,
            Transactions::Ynab { since } => get_ynab_transactions(&config, since).await?,
        },
        Commands::Budgets => get_ynab_budgets(&config).await?,
        Commands::Balance(x) => match x {
            Balance::Up {
                in_path,
                out_path,
                since,
                until,
            } => {
                up_balance(
                    &config,
                    in_path.as_deref(),
                    out_path.as_deref(),
                    since,
                    until,
                )
                .await?
            }
            Balance::Ynab {
                in_path,
                since,
                until,
            } => ynab_balance(&in_path, since, until).await?,
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
    in_path: Option<&Path>,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
    dry_run: Option<bool>,
) -> Result<()> {
    up_ynab::sync(SyncArgs {
        config,
        in_path,
        since,
        until,
        dry_run,
    })
    .await
}

async fn up_balance(
    config: &Config,
    in_path: Option<&Path>,
    out_path: Option<&Path>,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<()> {
    up_ynab::up_balance(config, in_path, out_path, since, until).await
}

async fn ynab_balance(
    in_path: &Path,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<()> {
    up_ynab::ynab_balance(in_path, since, until)
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
