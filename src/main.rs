mod up;

use std::env;

use color_eyre::eyre::Result;

const UP_API_URL_BASE: &str = "https://api.up.com.au/api";
const UP_API_VERSION: &str = "v1";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let url = format!("{UP_API_URL_BASE}/{UP_API_VERSION}/transactions");
    let api_key = env::var("UP_TOKEN")?;
    let client = reqwest::Client::new();
    let foo = client
        .get(url)
        .bearer_auth(api_key)
        .body("page[size]=10")
        .send()
        .await?;
    let s = foo.text().await?;
    dbg!(s);

    Ok(())
}
