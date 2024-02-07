use std::collections::HashMap;

use color_eyre::eyre::{eyre, ContextCompat, Result};
use itertools::Itertools;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    api::ynab,
    frontend::{cli, config::Config, run::Run},
    model::{Account, Transaction},
};

pub type Args = cli::sync::Args;

pub async fn sync(config: &Config, args: Args) -> Result<()> {
    let ynab_client = ynab::Client::new(&config.ynab.api_token);

    info!("starting up to ynab sync...");
    let run = if let Some(in_path) = args.in_path {
        Run::read(in_path)?
    } else {
        Run::fetch(config, args.since, args.until).await?
    };

    let budget_id = config
        .ynab
        .budget_id
        .as_ref()
        .map(|x| Uuid::parse_str(x))
        .wrap_err("missing budget id")??;
    let budget = run
        .ynab_budgets
        .wrap_err("missing ynab budgets")?
        .iter()
        .find(|x| x.id == budget_id)
        .cloned()
        .wrap_err("failed to find budget with id: `{budget_id}`")?;

    let accounts = Account::identify(
        &run.up_accounts.unwrap_or_default(),
        &run.ynab_accounts.unwrap_or_default(),
    )?;

    let ynab_transactions = run
        .ynab_transactions
        .map(|x| {
            x.into_iter()
                .map(|x| x.to_transaction(&budget, &accounts))
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?;

    let transactions = run
        .up_transactions
        .unwrap_or_default()
        .into_iter()
        .map(|x| x.to_transaction(&accounts))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|x| x.is_normalized())
        .collect::<Vec<_>>();

    let missing_transactions = ynab_transactions
        .as_ref()
        .map(|ynab_transactions| missing_transactions(&transactions, ynab_transactions))
        .unwrap_or_default();

    let not_eq_transactions = ynab_transactions
        .as_ref()
        .map(|ynab_transactions| not_eq_transactions(&transactions, ynab_transactions))
        .unwrap_or_default();

    if missing_transactions.is_empty() {
        info!("all up transactions exist in ynab!",);
    } else {
        info!(
            "found {} missing up transactions in ynab",
            missing_transactions.len()
        );
    }

    if not_eq_transactions.is_empty() {
        info!("all up transactions are unmodified in ynab!",);
    } else {
        info!(
            "found {} up transactions modified in ynab",
            not_eq_transactions.len()
        );
    }

    let new_ynab_transactions = missing_transactions
        .into_iter()
        .map(|x| x.clone().to_ynab())
        .inspect(|x| {
            if let Err(e) = x {
                error!("failed to convert to new ynab transaction: {e}")
            }
        })
        .collect::<Result<Vec<_>>>()?;

    if new_ynab_transactions.is_empty() {
        info!("nothing to do, stopping...");
        return Ok(());
    } else if args.dry_run {
        info!("dry run, skipping creating ynab transactions");
        return Ok(());
    }

    info!(
        "creating ynab {} transactions...",
        new_ynab_transactions.len()
    );
    let num_transactions = new_ynab_transactions.len();
    let response = ynab_client
        .new_transactions()
        .budget_id(budget_id)
        .transactions(new_ynab_transactions)
        .send()
        .await?;

    let num_missing =
        num_transactions - response.transactions.as_ref().unwrap_or(&Vec::new()).len();
    if num_missing != 0 {
        error!("failed to create {num_missing} transactions");
    }

    if let Some(duplicate_ids) = response.duplicate_import_ids
        && !duplicate_ids.is_empty()
    {
        return Err(eyre!(
            "found duplicate transaction ids: {}",
            duplicate_ids.iter().join(", ")
        ));
    }

    Ok(())
}

fn missing_transactions<'a>(
    source_transactions: &'a [Transaction],
    remote_transactions: &'a [Transaction],
) -> Vec<&'a Transaction> {
    let source_transactions_by_id = source_transactions
        .iter()
        .map(|x| (x.id.as_str(), x))
        .collect::<HashMap<&str, &Transaction>>();

    let remote_transactions_by_id = remote_transactions
        .iter()
        .filter(|x| x.imported_id.is_some())
        .map(|x| (x.imported_id.as_deref().unwrap(), x))
        .collect::<HashMap<&str, &Transaction>>();

    let missing_transactions = source_transactions_by_id
        .keys()
        .map(|k| (k, remote_transactions_by_id.get(k)))
        .filter(|(_, v)| v.is_none())
        .map(|(k, _)| source_transactions_by_id.get(k).unwrap())
        .copied()
        .collect::<Vec<_>>();

    missing_transactions
}

fn not_eq_transactions<'a>(
    source_transactions: &'a [Transaction],
    remote_transactions: &'a [Transaction],
) -> Vec<&'a Transaction> {
    let source_transactions_by_id = source_transactions
        .iter()
        .map(|x| (x.id.as_str(), x))
        .collect::<HashMap<&str, &Transaction>>();

    let remote_transactions_by_id = remote_transactions
        .iter()
        .filter(|x| x.imported_id.is_some())
        .map(|x| (x.imported_id.as_deref().unwrap(), x))
        .collect::<HashMap<&str, &Transaction>>();

    let not_eq_transactions = source_transactions_by_id
        .iter()
        .map(|(k, v)| (k, (v, remote_transactions_by_id.get(k))))
        .filter(|(_, (a, b))| b.map(|b| !a.is_equivalent(b)).unwrap_or_default())
        .map(|(k, _)| source_transactions_by_id.get(k).unwrap())
        .copied()
        .collect::<Vec<_>>();

    not_eq_transactions
}
