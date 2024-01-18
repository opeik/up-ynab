use std::pin::Pin;

use color_eyre::eyre::Result;
use futures::{
    stream::{self},
    Stream,
};
use up::{
    apis::{
        accounts_api::{AccountsGetError, AccountsGetParams},
        configuration::Configuration,
        transactions_api::{TransactionsGetError, TransactionsGetParams},
        Error,
    },
    models::{
        AccountResource, ListAccountsResponse, ListTransactionsResponse, TransactionResource,
    },
};

#[derive(Clone)]
pub struct Client {
    config: Configuration,
}

macro_rules! new_paginated_endpoint {
    ($name:ident, $page_fn:ident, $T:ty, $E:ty) => {
        pub fn $name(&self) -> Pin<Box<impl Stream<Item = Result<$T, Error<$E>>> + '_>> {
            use tracing::info;

            struct Page {
                data: <Vec<$T> as IntoIterator>::IntoIter,
                next: Option<String>,
                count: usize,
            }

            let page = Page {
                data: Vec::new().into_iter(),
                next: None,
                count: 0,
            };

            let items = stream::try_unfold(page, move |mut page: Page| async move {
                if let Some(x) = page.data.next() {
                    return Ok(Some((x, page)));
                } else if page.next.is_none() && page.count > 0 {
                    return Ok(None);
                }

                info!("fetching page {}...", page.count);
                let response = self.$page_fn(page.next.as_deref()).await?;
                page.data = response.data.into_iter();
                page.next = response.links.next;
                page.count += 1;
                Ok(page.data.next().map(|x| (x, page)))
            });

            Box::pin(items)
        }
    };
}

impl Client {
    const PAGE_SIZE: i32 = 100;

    pub fn new(api_token: &str) -> Self {
        Self {
            config: Configuration {
                bearer_access_token: Some(api_token.to_owned()),
                ..Default::default()
            },
        }
    }

    new_paginated_endpoint!(
        transactions,
        transactions_page,
        TransactionResource,
        TransactionsGetError
    );

    new_paginated_endpoint!(accounts, accounts_page, AccountResource, AccountsGetError);

    async fn transactions_page(
        &self,
        page: Option<&str>,
    ) -> Result<ListTransactionsResponse, Error<TransactionsGetError>> {
        if let Some(page) = page {
            up::apis::util::get_page::<ListTransactionsResponse, TransactionsGetError>(
                &self.config,
                page,
            )
            .await
        } else {
            up::apis::transactions_api::transactions_get(
                &self.config,
                TransactionsGetParams {
                    page_size: Some(Self::PAGE_SIZE),
                    filter_status: None,
                    filter_since: None,
                    filter_until: None,
                    filter_category: None,
                    filter_tag: None,
                },
            )
            .await
        }
    }

    async fn accounts_page(
        &self,
        page: Option<&str>,
    ) -> Result<ListAccountsResponse, Error<AccountsGetError>> {
        if let Some(page) = page {
            up::apis::util::get_page::<ListAccountsResponse, AccountsGetError>(&self.config, page)
                .await
        } else {
            up::apis::accounts_api::accounts_get(
                &self.config,
                AccountsGetParams {
                    page_size: Some(Self::PAGE_SIZE),
                    filter_type: None,
                    filter_ownership: None,
                },
            )
            .await
        }
    }
}
