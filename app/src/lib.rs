#![feature(let_chains)]
#![warn(
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic,
    clippy::unwrap_used
)]
#![allow(clippy::missing_errors_doc, clippy::wildcard_imports)]

pub mod api;
pub mod cmd;
pub mod frontend;
pub mod model;

pub type YnabBudget = ynab_client::models::BudgetSummary;
pub use color_eyre::eyre::{Error, Result};
