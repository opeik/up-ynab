use std::path::PathBuf;

use chrono::{DateTime, Utc};
use clap::Parser;
use color_eyre::eyre::Result;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use futures::{StreamExt, TryStreamExt};
use itertools::Itertools;
use tracing::{error, info};
use up_client::apis::transactions_api::TransactionsGetParams;
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

    match cli.command {
        Commands::GetUpAccounts => get_up_accounts(&config).await?,
        Commands::GetUpTransactions { from, until } => {
            get_up_transactions(&config, from, until).await?
        }
        Commands::GetYnabAccounts => get_ynab_accounts(&config).await?,
        Commands::GetYnabBudgets => get_ynab_budgets(&config).await?,
        Commands::GetYnabTransactions { from } => get_ynab_transactions(&config, from).await?,
        Commands::Sync { from, until } => sync(&config, from, until).await?,
    }

    Ok(())
}

async fn get_up_accounts(config: &Config) -> Result<()> {
    let up_client = up::Client::new(&config.up.api_token);

    info!("fetching up accounts...");
    let mut accounts = up_client.accounts(None);
    while let Some(account) = accounts.try_next().await? {
        info!("{}: {}", account.id, account.attributes.display_name);
    }

    Ok(())
}

async fn get_up_transactions(
    config: &Config,
    from: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
) -> Result<()> {
    let up_client = up::Client::new(&config.up.api_token);
    let accounts = config.account.clone().unwrap_or_default();

    info!("fetching up transactions...");
    let mut transactions = up_client
        .transactions(Some(TransactionsGetParams {
            page_size: Some(100),
            filter_since: from.map(|x| x.to_rfc3339()),
            filter_until: until.map(|x| x.to_rfc3339()),
            filter_status: None,
            filter_category: None,
            filter_tag: None,
        }))
        .map_ok(|x| Transaction::from_up(x, &accounts));

    while let Some(transaction) = transactions.next().await {
        let transaction = transaction??;
        info!("{transaction:?}");
    }

    Ok(())
}

async fn get_ynab_transactions(config: &Config, from: Option<DateTime<Utc>>) -> Result<()> {
    let ynab_client = ynab::Client::new(&config.ynab.api_token);
    let accounts = config.account.clone().unwrap_or_default();

    info!("fetching ynab transactions...");
    let transactions = ynab_client
        .transactions(&config.ynab.budget_id, from)
        .await?;
    // .map(|transactions| {
    //     transactions
    //         .data
    //         .transactions
    //         .into_iter()
    //         .map(|x| Transaction::from_ynab(x, &accounts))
    //         .collect::<Result<Vec<_>>>()
    // })??;

    for transaction in transactions.data.transactions {
        info!("{transaction:?}");
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
    let response = ynab_client.budgets().await?;
    for budget in response.data.budgets {
        info!("{}: {}", budget.id, budget.name);
    }

    Ok(())
}

async fn sync(
    config: &Config,
    from: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
) -> Result<()> {
    let up_client = up::Client::new(&config.up.api_token);
    let _ynab_client = ynab::Client::new(&config.ynab.api_token);
    let accounts = config.account.clone().unwrap_or_default();

    info!("fetching up transactions...");
    let args = Some(TransactionsGetParams {
        page_size: Some(100),
        filter_since: from.map(|x| x.to_rfc3339()),
        filter_until: until.map(|x| x.to_rfc3339()),
        filter_status: None,
        filter_category: None,
        filter_tag: None,
    });

    let mut up_transactions = up_client.transactions(args).chunks(100);

    while let Some(chunk) = up_transactions.next().await {
        let (oks, errs): (Vec<_>, Vec<_>) = chunk.into_iter().partition_result();
        let transactions = oks
            .into_iter()
            .map(|x| Transaction::from_up(x, &accounts).and_then(|x| x.to_ynab()))
            .collect::<Result<Vec<_>>>()?;

        for e in errs {
            error!("failed to get transaction: {e}");
        }

        info!("creating ynab transactions...");
        // info!("{transactions:#?}");
        // let _response = ynab_client
        //     .new_transactions(&config.ynab.budget_id, oks.as_slice())
        //     .await?;
        // debug!("{response:#?}");
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
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
