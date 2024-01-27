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
    let run_path = cli.run_path.as_deref();

    match cli.command {
        Commands::SyncTransactions { since, until } => {
            sync(&config, run_path, since, until).await?
        }
        Commands::GetUpAccounts => get_up_accounts(&config).await?,
        Commands::GetUpTransactions { since, until } => {
            get_up_transactions(&config, since, until).await?
        }
        Commands::GetYnabAccounts => get_ynab_accounts(&config).await?,
        Commands::GetYnabBudgets => get_ynab_budgets(&config).await?,
        Commands::GetYnabTransactions { since } => get_ynab_transactions(&config, since).await?,
    }

    Ok(())
}

async fn get_up_accounts(config: &Config) -> Result<()> {
    up_ynab::fetch_up_accounts(config).await?;
    Ok(())
}

async fn get_up_transactions(
    config: &Config,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<()> {
    up_ynab::fetch_up_transactions(config, since, until).await?;
    Ok(())
}

async fn get_ynab_transactions(
    config: &Config,
    since: Option<DateTime<FixedOffset>>,
) -> Result<()> {
    up_ynab::fetch_ynab_transactions(config, since).await?;
    Ok(())
}

async fn get_ynab_accounts(config: &Config) -> Result<()> {
    up_ynab::fetch_up_accounts(config).await?;
    Ok(())
}

async fn get_ynab_budgets(config: &Config) -> Result<()> {
    let ynab_client = ynab::Client::new(&config.ynab.api_token);

    info!("fetching ynab budgets...");
    let budgets = ynab_client.budgets().send().await?;
    for budget in budgets {
        info!("{}: {}", budget.id, budget.name);
    }

    Ok(())
}

async fn sync(
    config: &Config,
    run_path: Option<&Path>,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<()> {
    up_ynab::sync(config, run_path, since, until).await
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
