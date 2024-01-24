#[derive(Clone, Debug, serde::Deserialize)]
pub struct Up {
    pub api_token: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Ynab {
    pub api_token: String,
    pub budget_id: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub up: Up,
    pub ynab: Ynab,
}
