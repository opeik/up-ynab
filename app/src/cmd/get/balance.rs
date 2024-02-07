use tracing::info;

use crate::{
    frontend::{cli, Config, Run},
    model::{balance, Account, Transaction},
    Result,
};

pub type UpArgs = cli::get::balance::up::Args;
pub type YnabArgs = cli::get::balance::ynab::Args;

pub async fn up(config: &Config, args: UpArgs) -> Result<()> {
    let run = if let Some(in_path) = args.in_path {
        Run::read(in_path)?
    } else {
        Run::fetch(config, args.since, args.until).await?
    };

    let accounts = Account::identify(
        &run.up_accounts.unwrap_or_default(),
        &run.ynab_accounts.unwrap_or_default(),
    )?;

    let transactions = run
        .up_transactions
        .unwrap_or_default()
        .into_iter()
        .map(|x| Transaction::from_up(x.clone(), &accounts))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|x| x.is_normalized())
        .collect::<Vec<_>>();
    let balances = balance::running_balance(&transactions);

    for balance in &balances {
        if let Some(since) = args.since
            && balance.transaction.time <= since
        {
            continue;
        }

        if let Some(until) = args.until
            && balance.transaction.time >= until
        {
            continue;
        }

        info!("{balance}");
    }

    if let Some(out_path) = args.out_path {
        info!("writing balance CSV to `{}`", out_path.to_string_lossy());
        balance::write_balance_csv(&balances, out_path)?;
    }

    Ok(())
}

pub async fn ynab(_config: &Config, _args: YnabArgs) -> Result<()> {
    unimplemented!()
}
