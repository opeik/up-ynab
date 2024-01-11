mod up;

use std::env;

use color_eyre::eyre::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let up_api_token =
        up::ApiToken::new(env::var("UP_API_TOKEN").wrap_err("`UP_API_TOKEN` missing")?)?;

    let up_client = up::Client::builder().api_token(up_api_token).build()?;
    let transactions = up_client.transactions().page_size(10).send().await?;
    dbg!(transactions);

    Ok(())
}
