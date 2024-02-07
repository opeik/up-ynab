use color_eyre::eyre::ContextCompat;
use futures::{StreamExt, TryStreamExt};
use tracing::{error, info};

use crate::{
    api::{up, ynab},
    frontend::{cli, Config},
    Result, UpTransaction, YnabTransaction,
};

pub type UpArgs = cli::get::transaction::up::Args;
pub type YnabArgs = cli::get::transaction::ynab::Args;

pub async fn up(config: &Config, args: UpArgs) -> Result<Vec<UpTransaction>> {
    info!("fetching up transactions...");
    let up_client = up::Client::new(&config.up.api_token);
    let transactions = up_client
        .transactions()
        .filter_since(args.since)
        .filter_until(args.until)
        .send()?
        .inspect_err(|e| error!("failed to fetch transaction: {e}"))
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    Ok(transactions)
}

pub async fn ynab(config: &Config, args: YnabArgs) -> Result<Vec<YnabTransaction>> {
    info!("fetching ynab transactions...");
    let ynab_client = ynab::Client::new(&config.ynab.api_token);
    let budget_id = config
        .ynab
        .budget_id
        .as_ref()
        .wrap_err("missing budget id")?;
    let transactions = ynab_client
        .transactions()
        .budget_id(budget_id)
        .since_date(args.since)
        .send()
        .await?;
    Ok(transactions)
}
