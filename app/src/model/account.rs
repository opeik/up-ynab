use color_eyre::eyre::ContextCompat;
use tracing::{error, info};
use uuid::Uuid;

use crate::{Result, UpAccount, YnabAccount};

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
                let up_account_name = up_account.attributes.display_name.clone();
                let ynab_account = ynab_accounts
                    .iter()
                    .find(|x| x.name.trim() == up_account_name.trim())
                    .wrap_err(format!(
                        "failed to match up account `{up_account_name}` to ynab account"
                    ))?;
                Ok(Account {
                    name: up_account_name,
                    up_id: up_account.id.clone(),
                    ynab_id: ynab_account.id,
                    ynab_transfer_id: ynab_account
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
