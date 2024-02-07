use std::path::PathBuf;

use clap::Parser;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use up_ynab::{
    cmd,
    frontend::{
        cli,
        cli::{Cli, Commands},
        config::Config,
    },
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    install_tracing();

    let cli = Cli::parse();
    let config = Figment::new()
        .merge(Toml::file(
            cli.config.unwrap_or(PathBuf::from("config.toml")),
        ))
        .extract::<Config>()?;

    // TODO: replace this with a proc macro
    match cli.command {
        Commands::Sync(args) => {
            cmd::sync::sync(&config, args).await?;
        }
        Commands::Get(get) => match get {
            cli::get::Cmd::Account(account) => match account {
                cli::get::account::Cmd::Up => {
                    cmd::get::account::up(&config).await?;
                }
                cli::get::account::Cmd::Ynab => {
                    cmd::get::account::ynab(&config).await?;
                }
            },
            cli::get::Cmd::Transaction(transaction) => match transaction {
                cli::get::transaction::Cmd::Up(args) => {
                    cmd::get::transaction::up(&config, args).await?;
                }
                cli::get::transaction::Cmd::Ynab(args) => {
                    cmd::get::transaction::ynab(&config, args).await?;
                }
            },
            cli::get::Cmd::Balance(balance) => match balance {
                cli::get::balance::Cmd::Up(args) => {
                    cmd::get::balance::up(&config, args).await?;
                }
                cli::get::balance::Cmd::Ynab(args) => {
                    cmd::get::balance::ynab(&config, args).await?;
                }
            },
        },
    }

    Ok(())
}

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let fmt_layer = fmt::layer();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("up_ynab=debug"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
