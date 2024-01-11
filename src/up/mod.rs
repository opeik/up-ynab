use chrono::{DateTime, Utc};
use color_eyre::eyre::{Context, Result};
use nutype::nutype;
use url::Url;

pub mod types;
use types::*;

#[nutype(
    derive(Debug, Clone, AsRef, TryFrom),
    validate(regex = "up:yeah:[a-zA-Z0-9]+")
)]
pub struct ApiToken(String);

#[derive(Debug, Clone, derive_builder::Builder)]
#[builder(pattern = "owned")]
pub struct Client {
    api_token: ApiToken,
    #[builder(setter(skip), default = "reqwest::Client::new()")]
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, derive_builder::Builder)]
#[builder(pattern = "owned", setter(into), build_fn(private))]
pub struct TransactionParams<'a> {
    #[builder(private)]
    up_client: &'a Client,
    #[builder(default)]
    page_size: Option<i32>,
    #[builder(default)]
    filter_status: Option<TransactionStatus>,
    #[builder(default)]
    filter_since: Option<DateTime<Utc>>,
    #[builder(default)]
    filter_until: Option<DateTime<Utc>>,
    #[builder(default)]
    filter_category: Option<String>,
    #[builder(default)]
    filter_tag: Option<String>,
}

impl Client {
    const BASE_URL: &'static str = "https://api.up.com.au/api";
    const VERSION: &'static str = "v1";

    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    pub fn transactions(&self) -> TransactionParamsBuilder<'_> {
        TransactionParamsBuilder::default().up_client(self)
    }

    pub(crate) fn endpoint() -> Url {
        Url::parse(&format!("{}/{}/", Self::BASE_URL, Self::VERSION)).unwrap()
    }
}

impl<'a> TransactionParamsBuilder<'a> {
    pub async fn send(self) -> Result<Transactions> {
        let params = self
            .build()
            .wrap_err("failed to build transaction parameters")?;

        let transactions = params
            .up_client
            .http_client
            .get(Client::endpoint().join("transactions")?)
            .bearer_auth(params.up_client.api_token.as_ref())
            .query(&[("page[size]", params.page_size)])
            .query(&[("filter[status]", params.filter_status)])
            .query(&[("filter[since]", params.filter_since)])
            .query(&[("filter[until]", params.filter_until)])
            .query(&[("filter[category]", params.filter_category)])
            .query(&[("filter[tag]", params.filter_tag)])
            .send()
            .await?
            .json::<Transactions>()
            .await?;

        Ok(transactions)
    }
}
