#![feature(let_chains)]

pub mod api;
pub mod cmd;
pub mod frontend;
pub mod model;

// TODO: make newtypes
pub type UpAccount = up_client::models::AccountResource;
pub type YnabAccount = ynab_client::models::Account;
pub type YnabBudget = ynab_client::models::BudgetSummary;
pub use color_eyre::eyre::Result;
