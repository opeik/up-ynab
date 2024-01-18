pub mod cli;
pub mod config;
pub mod convert;
pub mod up;
pub mod ynab;

use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::{eyre, Result};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use futures::{future, StreamExt, TryStreamExt};
use itertools::Itertools;
use tracing::{debug, error, info};

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

    match cli.command {
        Commands::GetUpAccounts => get_up_accounts(&config).await?,
        Commands::GetUpTransactions => get_up_transactions(&config).await?,
        Commands::GetYnabAccounts => get_ynab_accounts(&config).await?,
        Commands::GetYnabBudgets => get_ynab_budgets(&config).await?,
        Commands::GetYnabTransactions => get_ynab_transactions(&config).await?,
        Commands::Sync => sync(&config).await?,
    }

    Ok(())
}

async fn get_up_accounts(config: &Config) -> Result<()> {
    let up_client = up::Client::new(&config.up.api_token);

    info!("fetching up accounts...");
    let mut accounts = up_client.accounts();
    while let Some(account) = accounts.try_next().await? {
        info!("{}: {}", account.id, account.attributes.display_name);
    }

    Ok(())
}

async fn get_up_transactions(config: &Config) -> Result<()> {
    let up_client = up::Client::new(&config.up.api_token);

    info!("fetching up transactions...");
    let mut transactions = up_client.transactions();
    while let Some(transaction) = transactions.try_next().await? {
        info!("{transaction:#?}");
    }

    Ok(())
}

async fn get_ynab_transactions(config: &Config) -> Result<()> {
    let ynab_client = ynab::Client::new(&config.ynab.api_token);

    info!("fetching ynab transactions...");
    let response = ynab_client.transactions(&config.ynab.budget_id).await?;
    for transaction in response.data.transactions {
        info!("{transaction:#?}");
    }

    Ok(())
}

async fn get_ynab_accounts(config: &Config) -> Result<()> {
    let ynab_client = ynab::Client::new(&config.ynab.api_token);

    info!("fetching ynab accounts...");
    let response = ynab_client.accounts(&config.ynab.budget_id).await?;
    for account in response.data.accounts {
        info!(
            "{}\nid: {}\ntransfer_id: {}",
            account.name,
            account.id,
            account.transfer_payee_id.map(|x| x.to_string()).unwrap(),
        );
    }

    Ok(())
}

async fn get_ynab_budgets(config: &Config) -> Result<()> {
    let ynab_client = ynab::Client::new(&config.ynab.api_token);

    info!("fetching ynab budgets...");
    let repsonse = ynab_client.budgets().await?;
    for budget in repsonse.data.budgets {
        info!("{}: {}", budget.id, budget.name);
    }

    Ok(())
}

async fn sync(config: &Config) -> Result<()> {
    let up_client = up::Client::new(&config.up.api_token);
    let ynab_client = ynab::Client::new(&config.ynab.api_token);

    info!("fetching up transactions...");
    let mut up_transactions = up_client
        .transactions()
        .map_ok(|x| convert::to_ynab_transaction(&x, config))
        .chunks(100);

    while let Some(chunk) = up_transactions.next().await {
        let (oks, errs): (Vec<_>, Vec<_>) = chunk.into_iter().partition_result();
        let oks = oks.into_iter().flatten().flatten().collect::<Vec<_>>();

        for e in errs {
            error!("failed to get transaction: {e}");
        }

        info!("creating ynab transactions...");
        let response = ynab_client
            .new_transactions(&config.ynab.budget_id, oks.as_slice())
            .await?;
        debug!("{response:#?}");
    }

    Ok(())
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
        .with(fmt_layer.with_target(true))
        .with(ErrorLayer::default())
        .init();
}
