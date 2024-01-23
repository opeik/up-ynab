#![feature(let_chains)]
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset};
use clap::Parser;
use color_eyre::eyre::{eyre, ContextCompat, Result};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use futures::{StreamExt, TryStreamExt};
use itertools::Itertools;
use money2::Decimal;
use tracing::info;
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
        Commands::Setup => todo!(),
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
    from: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
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
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Vec<_>>();

    #[derive(serde::Serialize)]
    struct Record {
        date_time: String,
        amount: String,
        round_up: Option<String>,
        cash_back: Option<String>,
        final_amount: String,
        to_account: String,
        from_account: Option<String>,
        description: String,
        message: Option<String>,
        spending_balance: Decimal,
        bills_balance: Decimal,
        home_loan_deposit_balance: Decimal,
        rent_balance: Decimal,
        emergency_balance: Decimal,
        home_balance: Decimal,
    }

    let mut spending_balance = Decimal::default();
    let mut bills_balance = Decimal::default();
    let mut home_loan_deposit_balance = Decimal::default();
    let mut rent_balance = Decimal::default();
    let mut emergency_balance = Decimal::default();
    let mut home_balance = Decimal::default();
    let mut csv = csv::Writer::from_path("out.csv")?;

    for transaction in transactions.into_iter().rev() {
        let transaction = transaction?;
        let attrs = &transaction.attributes;
        let x = Transaction::from_up(transaction.clone(), &accounts)?;

        // god help me
        match &x.kind {
            Kind::Expense { to, from_name: _ } => match to.name.as_str() {
                "Spending" => spending_balance += x.amount.amount,
                "Bills" => bills_balance += x.amount.amount,
                "Home loan deposit" => home_loan_deposit_balance += x.amount.amount,
                "Rent" => rent_balance += x.amount.amount,
                "Emergency" => emergency_balance += x.amount.amount,
                "Home" => home_balance += x.amount.amount,
                _ => {}
            },
            Kind::Transfer { to, from } => match to.name.as_str() {
                "Spending" => spending_balance += x.amount.amount,
                "Bills" => bills_balance += x.amount.amount,
                "Home loan deposit" => home_loan_deposit_balance += x.amount.amount,
                "Rent" => rent_balance += x.amount.amount,
                "Emergency" => emergency_balance += x.amount.amount,
                "Home" => home_balance += x.amount.amount,
                _ => {}
            },
        };

        let record = Record {
            date_time: attrs.created_at.clone(),
            amount: attrs.amount.value.clone(),
            round_up: attrs.round_up.clone().map(|x| x.amount.value),
            cash_back: attrs.cashback.clone().map(|x| x.amount.value),
            to_account: match &x.kind {
                Kind::Expense { to, from_name: _ } => to.name.clone(),
                Kind::Transfer { to, from: _ } => to.name.clone(),
            },
            from_account: match &x.kind {
                Kind::Expense { to: _, from_name } => None,
                Kind::Transfer { to: _, from } => Some(from.name.clone()),
            },
            message: attrs.message.clone(),
            description: attrs.description.clone(),
            final_amount: x.amount.amount.to_string(),
            spending_balance,
            bills_balance,
            home_balance,
            home_loan_deposit_balance,
            rent_balance,
            emergency_balance,
        };

        csv.serialize(record)?;
        info!("{transaction:?}");
        info!("{x:?}");
    }

    csv.flush()?;

    Ok(())
}

async fn get_ynab_transactions(config: &Config, from: Option<DateTime<FixedOffset>>) -> Result<()> {
    let ynab_client = ynab::Client::new(&config.ynab.api_token);
    let budget_id = config
        .ynab
        .budget_id
        .as_ref()
        .wrap_err("missing budget id")?;

    info!("fetching ynab transactions...");
    let transactions = ynab_client.transactions(budget_id, from).await?;
    // .map(|transactions| {
    //     transactions
    //         .data
    //         .transactions
    //         .into_iter()
    //         .map(|x| Transaction::from_ynab(x, &accounts))
    //         .collect::<Result<Vec<_>>>()
    // })??;

    let l = transactions.data.transactions.len();
    info!("found {} transactions", l);

    for transaction in transactions.data.transactions {
        info!("{transaction:?}");
    }

    Ok(())
}

async fn get_ynab_accounts(config: &Config) -> Result<()> {
    let ynab_client = ynab::Client::new(&config.ynab.api_token);
    let budget_id = config
        .ynab
        .budget_id
        .as_ref()
        .wrap_err("missing budget id")?;

    info!("fetching ynab accounts...");
    let response = ynab_client.accounts(budget_id).await?;
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
    from: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
) -> Result<()> {
    let up_client = up::Client::new(&config.up.api_token);
    let ynab_client = ynab::Client::new(&config.ynab.api_token);
    let accounts = config.account.clone().unwrap_or_default();
    let budget_id = config
        .ynab
        .budget_id
        .as_ref()
        .wrap_err("missing budget id")?;

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

    fn is_outgoing_transfer(x: &Transaction) -> bool {
        let msg_matches = x
            .msg
            .as_ref()
            .map(|s| {
                s.contains("Transfer to ")
                    || s.contains("Cover to ")
                    || s.contains("Quick save transfer to")
                    || s.contains("Forward to ")
            })
            .unwrap_or(false);

        msg_matches
    }

    while let Some(chunk) = up_transactions.next().await {
        let (oks, errs): (Vec<_>, Vec<_>) = chunk.into_iter().partition_result();
        let transactions = oks
            .into_iter()
            .flat_map(|x| Transaction::from_up(x, &accounts))
            .filter(|x| !is_outgoing_transfer(x))
            .inspect(|x| info!("{x:?}"))
            .map(|x| x.to_ynab())
            .collect::<Result<Vec<_>>>()?;

        if !errs.is_empty() {
            return Err(eyre!("failed to get transactions: {:?}", errs));
        }

        info!("creating ynab transactions...");
        let response = ynab_client
            .new_transactions(budget_id, &transactions)
            .await?;

        let num_missing = transactions.len()
            - response
                .data
                .transactions
                .as_ref()
                .unwrap_or(&Vec::new())
                .len();

        if num_missing != 0 {
            return Err(eyre!("failed to create {num_missing} transactions"));
        }

        if let Some(duplicate_ids) = response.data.duplicate_import_ids
            && !duplicate_ids.is_empty()
        {
            return Err(eyre!(
                "found duplicate transaction ids: {}",
                duplicate_ids.iter().join(", ")
            ));
        }
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
