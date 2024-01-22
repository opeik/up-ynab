use chrono::{DateTime, FixedOffset, Utc};
use color_eyre::eyre::{Context, Result};
use ynab_client::{
    apis::{
        accounts_api::GetAccountsParams,
        budgets_api::GetBudgetsParams,
        configuration::Configuration,
        transactions_api::{
            CreateTransactionParams, DeleteTransactionParams, GetTransactionsParams,
        },
    },
    models::{
        AccountsResponse, BudgetSummaryResponse, PostTransactionsWrapper, SaveTransaction,
        SaveTransactionsResponse, TransactionResponse, TransactionsResponse,
    },
};

pub struct Client {
    config: Configuration,
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

    pub async fn accounts(&self, budget_id: &str) -> Result<AccountsResponse> {
        ynab_client::apis::accounts_api::get_accounts(
            &self.config,
            GetAccountsParams {
                budget_id: budget_id.to_owned(),
                last_knowledge_of_server: None,
            },
        )
        .await
        .wrap_err("failed to get accounts")
    }

    pub async fn budgets(&self) -> Result<BudgetSummaryResponse> {
        ynab_client::apis::budgets_api::get_budgets(
            &self.config,
            GetBudgetsParams {
                include_accounts: None,
            },
        )
        .await
        .wrap_err("failed to get budgets")
    }

    pub async fn transactions(
        &self,
        budget_id: &str,
        from: Option<DateTime<FixedOffset>>,
    ) -> Result<TransactionsResponse> {
        ynab_client::apis::transactions_api::get_transactions(
            &self.config,
            GetTransactionsParams {
                budget_id: budget_id.to_owned(),
                since_date: from.map(|x| x.to_rfc3339()),
                r#type: None,
                last_knowledge_of_server: None,
            },
        )
        .await
        .wrap_err("failed to get transactions")
    }

    pub async fn new_transactions(
        &self,
        budget_id: &str,
        transactions: &[SaveTransaction],
    ) -> Result<SaveTransactionsResponse> {
        ynab_client::apis::transactions_api::create_transaction(
            &self.config,
            CreateTransactionParams {
                budget_id: budget_id.to_owned(),
                data: PostTransactionsWrapper {
                    transaction: None,
                    transactions: Some(transactions.to_vec()),
                },
            },
        )
        .await
        .wrap_err("failed to create transactions")
    }

    pub async fn remove_transaction(
        &self,
        budget_id: &str,
        transaction_id: &str,
    ) -> Result<TransactionResponse> {
        ynab_client::apis::transactions_api::delete_transaction(
            &self.config,
            DeleteTransactionParams {
                budget_id: budget_id.to_owned(),
                transaction_id: transaction_id.to_owned(),
            },
        )
        .await
        .wrap_err("failed to remove transaction")
    }
}
