#![feature(let_chains)]

pub mod api;
pub mod cmd;
pub mod frontend;
pub mod model;

pub type YnabBudget = ynab_client::models::BudgetSummary;
pub use color_eyre::eyre::{Error, Result};
