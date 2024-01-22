use std::pin::Pin;

use color_eyre::eyre::Result;
use futures::{
    stream::{self},
    Stream,
};
use up_client::{
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

macro_rules! stream_pages_impl {
    ($name:ident, $page_fn:ident, $T:ty, $A:ty, $E:ty) => {
        pub fn $name(
            &self,
            args: Option<$A>,
        ) -> Pin<Box<impl Stream<Item = Result<$T, Error<$E>>> + '_>> {
            use tracing::debug;

            struct State {
                data: <Vec<$T> as IntoIterator>::IntoIter,
                next: Option<String>,
                count: usize,
                args: Option<$A>,
            }

            let state = State {
                data: Vec::new().into_iter(),
                next: None,
                count: 0,
                args,
            };

            let items = stream::try_unfold(state, move |mut state: State| async move {
                if let Some(x) = state.data.next() {
                    return Ok(Some((x, state)));
                } else if state.next.is_none() && state.count > 0 {
                    return Ok(None);
                }

                debug!("fetching page {}", state.count);
                let response = self
                    .$page_fn(state.next.as_deref(), state.args.clone())
                    .await?;
                state.data = response.data.into_iter();
                state.next = response.links.next;
                state.count += 1;
                Ok(state.data.next().map(|x| (x, state)))
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

    stream_pages_impl!(
        transactions,
        transactions_page,
        TransactionResource,
        TransactionsGetParams,
        TransactionsGetError
    );

    stream_pages_impl!(
        accounts,
        accounts_page,
        AccountResource,
        AccountsGetParams,
        AccountsGetError
    );

    async fn transactions_page(
        &self,
        page: Option<&str>,
        params: Option<TransactionsGetParams>,
    ) -> Result<ListTransactionsResponse, Error<TransactionsGetError>> {
        if let Some(page) = page {
            up_client::apis::util::get_page::<ListTransactionsResponse, TransactionsGetError>(
                &self.config,
                page,
            )
            .await
        } else {
            up_client::apis::transactions_api::transactions_get(
                &self.config,
                params.unwrap_or(TransactionsGetParams {
                    page_size: Some(Self::PAGE_SIZE),
                    filter_status: None,
                    filter_since: None,
                    filter_until: None,
                    filter_category: None,
                    filter_tag: None,
                }),
            )
            .await
        }
    }

    async fn accounts_page(
        &self,
        page: Option<&str>,
        params: Option<AccountsGetParams>,
    ) -> Result<ListAccountsResponse, Error<AccountsGetError>> {
        if let Some(page) = page {
            up_client::apis::util::get_page::<ListAccountsResponse, AccountsGetError>(
                &self.config,
                page,
            )
            .await
        } else {
            up_client::apis::accounts_api::accounts_get(
                &self.config,
                params.unwrap_or(AccountsGetParams {
                    page_size: Some(Self::PAGE_SIZE),
                    filter_type: None,
                    filter_ownership: None,
                }),
            )
            .await
        }
    }
}
