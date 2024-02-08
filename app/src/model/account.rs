use color_eyre::eyre::ContextCompat;
use tracing::{error, info};
use uuid::Uuid;

use crate::Result;

pub type UpAccountInner = up_client::models::AccountResource;
pub type YnabAccountInner = ynab_client::models::Account;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct UpAccount(pub UpAccountInner);

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct YnabAccount(pub YnabAccountInner);

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
pub struct Account {
    pub name: String,
    pub up_id: String,
    pub ynab_id: Uuid,
    pub ynab_transfer_id: Uuid,
}

impl Account {
    pub fn identify(up_accounts: &[UpAccount], ynab_accounts: &[YnabAccount]) -> Result<Vec<Self>> {
        let accounts = up_accounts
            .iter()
            .map(|up_account| {
                let up_account_name = up_account.0.attributes.display_name.clone();
                let ynab_account = ynab_accounts
                    .iter()
                    .find(|x| x.0.name.trim() == up_account_name.trim())
                    .wrap_err(format!(
                        "failed to match up account `{up_account_name}` to ynab account"
                    ))?;
                Ok(Account {
                    name: up_account_name,
                    up_id: up_account.0.id.clone(),
                    ynab_id: ynab_account.0.id,
                    ynab_transfer_id: ynab_account
                        .0
                        .transfer_payee_id
                        .wrap_err("missing ynab transfer id")?,
                })
            })
            .inspect(|x: &Result<Account>| {
                if let Err(e) = x {
                    error!("{e}");
                };
            })
            .flatten()
            .collect::<Vec<Account>>();
        info!("matched {} accounts", accounts.len());
        Ok(accounts)
    }
}
