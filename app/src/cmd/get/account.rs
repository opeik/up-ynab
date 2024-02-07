use color_eyre::eyre::ContextCompat;
use futures::{StreamExt, TryStreamExt};
use tracing::{error, info};

use crate::{
    api::{up, ynab},
    frontend::config::Config,
    Result, UpAccount, YnabAccount,
};

pub async fn up(config: &Config) -> Result<Vec<UpAccount>> {
    info!("fetching up accounts...");
    let up_client = up::Client::new(&config.up.api_token);
    let accounts = up_client
        .accounts()
        .send()?
        .inspect_err(|e| error!("failed to fetch transaction: {e}"))
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    Ok(accounts)
}

pub async fn ynab(config: &Config) -> Result<Vec<YnabAccount>> {
    info!("fetching ynab accounts...");
    let ynab_client = ynab::Client::new(&config.ynab.api_token);
    let budget_id = config
        .ynab
        .budget_id
        .as_ref()
        .wrap_err("missing budget id")?;
    let accounts = ynab_client.accounts().budget_id(budget_id).send().await?;
    Ok(accounts)
}
