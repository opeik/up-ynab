use std::collections::HashMap;

use color_eyre::eyre::{ContextCompat, Result};
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
        .transpose()?
        .unwrap_or_default();

    let up_transactions = run
        .up_transactions
        .unwrap_or_default()
        .into_iter()
        .map(|x| x.to_transaction(&accounts))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|x| x.is_normalized())
        .collect::<Vec<_>>();

    let missing_transactions = find_missing_transactions(&up_transactions, &ynab_transactions);
    let modified_transactions = find_modified_transactions(&up_transactions, &ynab_transactions);

    if !missing_transactions.is_empty() {
        info!(
            "creating {} missing up transactions in ynab...",
            missing_transactions.len()
        );

        if !args.dry_run {
            let count = missing_transactions.len();
            let new_ynab_transactions = missing_transactions
                .into_iter()
                .cloned()
                .map(|x| x.to_new_ynab())
                .collect::<Result<Vec<_>>>()?;

            // TODO: check equality against returned transactions
            let response = ynab_client
                .new_transactions()
                .budget_id(budget_id)
                .transactions(new_ynab_transactions)
                .send()
                .await?;

            let num_failed = count - response.transaction_ids.len();
            if num_failed != 0 {
                error!("failed to create {} transactions", num_failed)
            }
        } else {
            info!("dry run, skipping...");
        }
    } else {
        info!("all up transactions exist in ynab!")
    }

    if !modified_transactions.is_empty() {
        info!(
            "updating {} modified up transactions in ynab...",
            modified_transactions.len()
        );

        if !args.dry_run {
            let count = modified_transactions.len();
            let updated_ynab_transactions = modified_transactions
                .into_iter()
                .map(|(source, remote)| {
                    let mut x = source.clone().to_update_ynab()?;
                    x.id = Some(remote.id.clone());
                    Ok(x)
                })
                .collect::<Result<Vec<_>>>()?;

            // TODO: check equality against returned transactions
            let response = ynab_client
                .update_transactions()
                .budget_id(budget_id)
                .transactions(updated_ynab_transactions)
                .send()
                .await?;

            let num_failed = count - response.transaction_ids.len();
            if num_failed != 0 {
                error!("failed to update {} transactions", num_failed)
            }
        } else {
            info!("dry run, skipping...");
        }
    } else {
        info!("all up transactions unmodified in ynab!")
    }

    info!("done!");
    Ok(())
}

fn find_missing_transactions<'a>(
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

fn find_modified_transactions<'a>(
    source_transactions: &'a [Transaction],
    remote_transactions: &'a [Transaction],
) -> Vec<(&'a Transaction, &'a Transaction)> {
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
        .map(|(_, (a, b))| (*a, *b.unwrap()))
        .collect::<Vec<_>>();

    not_eq_transactions
}
