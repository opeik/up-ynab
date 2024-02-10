use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use color_eyre::eyre::{eyre, Context, ContextCompat, Result};
use pretty_assertions::Comparison;
use tracing::error;
use uuid::Uuid;
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

#[derive(Debug, PartialEq)]
struct TransactionDiff {
    import_id: Option<String>,
    date: Option<String>,
    amount: Option<i64>,
    memo: Option<String>,
    account_id: Option<Uuid>,
    payee_name: Option<String>,
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
                        .map(|x| x.into_inner())
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
                    .map(|x| x.into_inner())
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
                .map(YnabAccount::new)
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
                .map(YnabTransaction::new)
                .collect::<Vec<_>>(),
        )
    }
}

macro_rules! check_response {
    ($transactions:expr, $response:expr, $msg:expr) => {
        let msg = $msg;
        let num_transactions = $transactions.len();
        if num_transactions != $response.transaction_ids.len() {
            return Err(eyre!(
                "failed to {msg} {} transactions",
                num_transactions - $response.transaction_ids.len()
            ));
        }

        if let Some(duplicate_import_ids) = &$response.duplicate_import_ids
            && !duplicate_import_ids.is_empty()
        {
            return Err(eyre!("attempted to {msg} duplicate transactions"));
        }

        let updated_transactions = $response
            .transactions
            .as_ref()
            .wrap_err("missing transactions in response")?;

        let transactions_by_id = $transactions
            .iter()
            .map(|x| {
                if let Some(Some(import_id)) = &x.import_id {
                    Ok((import_id.as_str(), x))
                } else {
                    Err(eyre!("missing import id"))
                }
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let new_transactions_by_id = updated_transactions
            .iter()
            .map(|x| {
                if let Some(Some(import_id)) = &x.import_id {
                    Ok((import_id.as_str(), x))
                } else {
                    Err(eyre!("missing import id"))
                }
            })
            .collect::<Result<HashMap<_, _>>>()?;

        for (id, transaction) in transactions_by_id {
            let new_transaction = new_transactions_by_id
                .get(id)
                .wrap_err(format!("transaction {id} failed to {msg}"))?;

            // TODO: reduce clones
            let a = TransactionDiff {
                import_id: transaction.import_id.clone().flatten(),
                date: transaction.date.clone(),
                amount: transaction.amount,
                memo: transaction.memo.clone().flatten(),
                account_id: transaction.account_id,
                payee_name: transaction.payee_name.clone().flatten(),
            };

            let b = TransactionDiff {
                import_id: new_transaction.import_id.clone().flatten(),
                date: Some(new_transaction.date.clone()),
                account_id: Some(new_transaction.account_id),
                amount: Some(new_transaction.amount),
                memo: new_transaction.memo.clone().flatten(),
                payee_name: new_transaction.payee_name.clone().flatten(),
            };

            if a != b {
                // TODO: probably should use a different crate for this
                error!(
                    "transaction mismatch in response:\n{}",
                    Comparison::new(&b, &a)
                );
                return Err(eyre!("transaction mismatch in response"));
            }
        }
    };
}

impl<'a> NewTransactionsParamsBuilder<'a> {
    pub async fn send(self) -> Result<models::SaveTransactionsResponseData> {
        let params = self.build().wrap_err("failed to build parameters")?;
        let transactions = params.transactions.clone();
        let response =
            *transactions_api::create_transaction(&params.client.config, params.into_api())
                .await
                .wrap_err("failed to create transactions")?
                .data;
        check_response!(transactions, response, "create");
        Ok(response)
    }
}

impl<'a> UpdateTransactionsParamsBuilder<'a> {
    pub async fn send(self) -> Result<models::SaveTransactionsResponseData> {
        let params = self.build().wrap_err("failed to build parameters")?;
        let transactions = params.transactions.clone();
        let response =
            *transactions_api::update_transactions(&params.client.config, params.into_api())
                .await
                .wrap_err("failed to create transactions")?
                .data;
        check_response!(transactions, response, "update");
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
