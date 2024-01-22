use uuid::Uuid;

use crate::Account;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Up {
    pub api_token: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Ynab {
    pub api_token: String,
    pub budget_id: String,
    pub account_id: Uuid,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub up: Up,
    pub ynab: Ynab,
    pub account: Option<Vec<Account>>,
}
