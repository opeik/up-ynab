use uuid::Uuid;

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
pub struct Account {
    pub name: String,
    pub up_id: String,
    pub ynab_id: Uuid,
    pub ynab_transfer_id: Uuid,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub up: Up,
    pub ynab: Ynab,
    pub account: Option<Vec<Account>>,
}
