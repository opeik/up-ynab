use chrono::DateTime;
use color_eyre::eyre::Result;
use up::models::TransactionResource;
use ynab::models::{SaveTransaction, TransactionClearedStatus};

use crate::{config::Account, Config};

#[derive(Debug, Copy, Clone)]
struct AccountMap<'a> {
    incoming: &'a Account,
    outgoing: &'a Account,
}

pub fn to_ynab_transaction(
    value: &TransactionResource,
    config: &Config,
) -> Result<Option<SaveTransaction>> {
    let amount = i64::from(value.attributes.amount.value_in_base_units * 10);
    let date = DateTime::parse_from_rfc3339(value.attributes.created_at.as_str())?.to_rfc3339();
    let account = match_accounts(value, config);

    // Up needs two transactions to transfer money between accounts (except for Round Ups?).
    // Since YNAB can do it in one, skip all "to" transactions.

    // let description = value.attributes.description.as_str();
    // if account.is_some()
    //     && (description.contains("Transfer to") || description.contains("Forward to"))
    // {
    //     return Ok(None);
    // }

    let ynab_transaction = if let Some(account) = account {
        SaveTransaction {
            account_id: Some(account.incoming.ynab_id),
            date: date.clone().into(),
            amount: Some(amount),
            payee_id: Some(Some(account.outgoing.ynab_transfer_id)),
            payee_name: None,
            category_id: None,
            memo: Some(Some(value.attributes.description.clone())),
            cleared: Some(TransactionClearedStatus::Cleared),
            approved: None,
            flag_color: None,
            import_id: None,
            subtransactions: None,
        }
    } else {
        SaveTransaction {
            account_id: Some(config.ynab.account_id),
            date: date.clone().into(),
            amount: Some(amount),
            payee_id: None,
            payee_name: Some(Some(value.attributes.description.clone())),
            category_id: None,
            memo: Some(value.attributes.message.clone()),
            cleared: Some(TransactionClearedStatus::Cleared),
            approved: None,
            flag_color: None,
            // import_id: Some(Some(format!("up_ynab:{}", value.id))),
            import_id: None,
            subtransactions: None,
        }
    };

    Ok(Some(ynab_transaction))
}

// this is awful
fn match_accounts<'a>(value: &TransactionResource, config: &'a Config) -> Option<AccountMap<'a>> {
    if let Some(accounts) = &config.account {
        let incoming = accounts
            .iter()
            .find(|x| x.up_id == value.relationships.account.data.id);

        let outgoing = accounts.iter().find(|x| {
            Some(&x.up_id)
                == value
                    .clone()
                    .relationships
                    .transfer_account
                    .data
                    .map(|x| x.id)
                    .as_ref()
        });

        match (incoming, outgoing) {
            (Some(incoming), Some(outgoing)) => Some(AccountMap { incoming, outgoing }),
            _ => None,
        }
    } else {
        None
    }
}
