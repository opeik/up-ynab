use chrono::{DateTime, FixedOffset};
use color_eyre::eyre::{eyre, Context, Result};
use itertools::Itertools;
use ynab_client::{
    apis::{accounts_api, budgets_api, configuration::Configuration, transactions_api},
    models,
};

use crate::model::{
    transaction::{NewYnabTransaction, UpdateYnabTransaction},
    YnabAccount, YnabTransaction,
};

#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) config: Configuration,
}

#[derive(Debug, Clone, derive_builder::Builder)]
#[builder(pattern = "owned", setter(into), build_fn(private))]
#[builder_struct_attr(must_use)]
pub struct GetAccountsParams<'a> {
    #[builder(private)]
    client: &'a Client,
    /// The id of the budget. `last-used` can be used to specify the last used budget and
    /// `default` can be used if default budget selection is enabled (see: https://api.ynab.com/#oauth-default-budget).
    budget_id: String,
    /// The starting server knowledge.  If provided, only entities that have changed since
    /// `last_knowledge_of_server` will be included.
    #[builder(default)]
    last_knowledge_of_server: Option<i64>,
}

#[derive(Debug, Clone, derive_builder::Builder)]
#[builder(pattern = "owned", setter(into), build_fn(private))]
#[builder_struct_attr(must_use)]
pub struct GetBudgetsParams<'a> {
    #[builder(private)]
    client: &'a Client,
    /// Whether to include the list of budget accounts
    #[builder(default)]
    pub include_accounts: Option<bool>,
}

#[derive(Debug, Clone, derive_builder::Builder)]
#[builder(pattern = "owned", setter(into), build_fn(private))]
#[builder_struct_attr(must_use)]
pub struct GetTransactionsParams<'a> {
    #[builder(private)]
    client: &'a Client,
    /// The id of the budget. `last-used` can be used to specify the last used budget and
    ///  `default` can be used if default budget selection is enabled (see: https://api.ynab.com/#oauth-default-budget).
    pub budget_id: String,
    /// If specified, only transactions on or after this date will be included.
    #[builder(default)]
    pub since_date: Option<DateTime<FixedOffset>>,
    /// If specified, only transactions of the specified type will be included. `uncategorized`
    /// and `unapproved` are currently supported.
    #[builder(default)]
    pub kind: Option<String>,
    /// The starting server knowledge.  If provided, only entities that have changed since
    /// `last_knowledge_of_server` will be included.
    #[builder(default)]
    pub last_knowledge_of_server: Option<i64>,
}

#[derive(Debug, Clone, derive_builder::Builder)]
#[builder(pattern = "owned", setter(into), build_fn(private))]
#[builder_struct_attr(must_use)]
pub struct NewTransactionsParams<'a> {
    #[builder(private)]
    client: &'a Client,
    /// The id of the budget. `last-used` can be used to specify the last used budget and
    /// `default` can be used if default budget selection is enabled (see: https://api.ynab.com/#oauth-default-budget).
    pub budget_id: String,
    /// The transactions to create.
    pub transactions: Vec<NewYnabTransaction>,
}

#[derive(Debug, Clone, derive_builder::Builder)]
#[builder(pattern = "owned", setter(into), build_fn(private))]
#[builder_struct_attr(must_use)]
pub struct UpdateTransactionsParams<'a> {
    #[builder(private)]
    client: &'a Client,
    /// The id of the budget. `last-used` can be used to specify the last used budget and
    /// `default` can be used if default budget selection is enabled (see: https://api.ynab.com/#oauth-default-budget).
    pub budget_id: String,
    /// The transactions to update.
    pub transactions: Vec<UpdateYnabTransaction>,
}

impl<'a> GetAccountsParams<'a> {
    fn into_api(self) -> accounts_api::GetAccountsParams {
        accounts_api::GetAccountsParams {
            budget_id: self.budget_id,
            last_knowledge_of_server: self.last_knowledge_of_server,
        }
    }
}

impl<'a> GetBudgetsParams<'a> {
    fn into_api(self) -> budgets_api::GetBudgetsParams {
        budgets_api::GetBudgetsParams {
            include_accounts: self.include_accounts,
        }
    }
}

impl<'a> GetTransactionsParams<'a> {
    fn into_api(self) -> transactions_api::GetTransactionsParams {
        transactions_api::GetTransactionsParams {
            budget_id: self.budget_id.to_owned(),
            since_date: self.since_date.map(|x| x.to_rfc3339()),
            r#type: self.kind,
            last_knowledge_of_server: self.last_knowledge_of_server,
        }
    }
}

impl<'a> NewTransactionsParams<'a> {
    fn into_api(self) -> transactions_api::CreateTransactionParams {
        transactions_api::CreateTransactionParams {
            budget_id: self.budget_id,
            data: models::PostTransactionsWrapper {
                transaction: None,
                transactions: Some(
                    self.transactions
                        .into_iter()
                        .map(|x| x.0)
                        .collect::<Vec<_>>(),
                ),
            },
        }
    }
}

impl<'a> UpdateTransactionsParams<'a> {
    fn into_api(self) -> transactions_api::UpdateTransactionsParams {
        transactions_api::UpdateTransactionsParams {
            budget_id: self.budget_id,
            data: models::PatchTransactionsWrapper {
                transactions: self
                    .transactions
                    .into_iter()
                    .map(|x| x.0)
                    .collect::<Vec<_>>(),
            },
        }
    }
}

impl<'a> GetAccountsParamsBuilder<'a> {
    pub async fn send(self) -> Result<Vec<YnabAccount>> {
        let params = self.build().wrap_err("failed to build parameters")?;
        Ok(
            accounts_api::get_accounts(&params.client.config, params.into_api())
                .await
                .wrap_err("failed to get accounts")?
                .data
                .accounts
                .into_iter()
                .map(YnabAccount)
                .collect::<Vec<_>>(),
        )
    }
}

impl<'a> GetBudgetsParamsBuilder<'a> {
    pub async fn send(self) -> Result<Vec<models::BudgetSummary>> {
        let params = self.build().wrap_err("failed to build parameters")?;
        Ok(
            budgets_api::get_budgets(&params.client.config, params.into_api())
                .await
                .wrap_err("failed to get budgets")?
                .data
                .budgets,
        )
    }
}

impl<'a> GetTransactionsParamsBuilder<'a> {
    pub async fn send(self) -> Result<Vec<YnabTransaction>> {
        let params = self.build().wrap_err("failed to build parameters")?;
        Ok(
            transactions_api::get_transactions(&params.client.config, params.into_api())
                .await
                .wrap_err("failed to get transactions")?
                .data
                .transactions
                .into_iter()
                .map(YnabTransaction)
                .collect::<Vec<_>>(),
        )
    }
}

impl<'a> NewTransactionsParamsBuilder<'a> {
    pub async fn send(self) -> Result<models::SaveTransactionsResponseData> {
        let params = self.build().wrap_err("failed to build parameters")?;
        let num_transactions = params.transactions.len();
        let response =
            *transactions_api::create_transaction(&params.client.config, params.into_api())
                .await
                .wrap_err("failed to create transactions")?
                .data;

        if num_transactions != response.transaction_ids.len() {
            return Err(eyre!(
                "failed to create {} transactions",
                num_transactions - response.transaction_ids.len()
            ));
        }

        if let Some(duplicate_import_ids) = &response.duplicate_import_ids
            && !duplicate_import_ids.is_empty()
        {
            return Err(eyre!(
                "attempted to create transactions with duplicate ids: {}",
                duplicate_import_ids
                    .iter()
                    .map(|x| format!("`{x}`"))
                    .join(", ")
            ));
        }

        Ok(response)
    }
}

impl<'a> UpdateTransactionsParamsBuilder<'a> {
    pub async fn send(self) -> Result<models::SaveTransactionsResponseData> {
        let params = self.build().wrap_err("failed to build parameters")?;
        let num_transactions = params.transactions.len();
        let response =
            *transactions_api::update_transactions(&params.client.config, params.into_api())
                .await
                .wrap_err("failed to create transactions")?
                .data;

        if num_transactions != response.transaction_ids.len() {
            return Err(eyre!(
                "failed to update {} transactions",
                num_transactions - response.transaction_ids.len()
            ));
        }

        if let Some(duplicate_import_ids) = &response.duplicate_import_ids
            && !duplicate_import_ids.is_empty()
        {
            return Err(eyre!("this should never happen",));
        }

        Ok(response)
    }
}

impl Client {
    pub fn new(api_token: &str) -> Self {
        Self {
            config: Configuration {
                bearer_access_token: Some(api_token.to_owned()),
                ..Default::default()
            },
        }
    }

    /// Returns all accounts.
    pub fn accounts(&self) -> GetAccountsParamsBuilder<'_> {
        GetAccountsParamsBuilder {
            client: Some(self),
            ..Default::default()
        }
    }

    /// Returns budgets list with summary information.
    pub fn budgets(&self) -> GetBudgetsParamsBuilder<'_> {
        GetBudgetsParamsBuilder {
            client: Some(self),
            ..Default::default()
        }
    }

    /// Returns budget transactions.
    pub fn transactions(&self) -> GetTransactionsParamsBuilder<'_> {
        GetTransactionsParamsBuilder {
            client: Some(self),
            ..Default::default()
        }
    }

    /// Creates a single transaction or multiple transactions.  If you provide a body containing
    /// a `transaction` object, a single transaction will be created and if you provide a body
    /// containing a `transactions` array, multiple transactions will be created.  Scheduled
    /// transactions cannot be created on this endpoint.
    pub fn new_transactions(&self) -> NewTransactionsParamsBuilder<'_> {
        NewTransactionsParamsBuilder {
            client: Some(self),
            ..Default::default()
        }
    }

    /// Updates multiple transactions.
    pub fn update_transactions(&self) -> UpdateTransactionsParamsBuilder<'_> {
        UpdateTransactionsParamsBuilder {
            client: Some(self),
            ..Default::default()
        }
    }
}
