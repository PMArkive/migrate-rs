use crate::config::{Config, ConfigError};
use crate::migrate::Migrator;
use crate::store::Store;
use clap::Parser;
use demostf_client::ApiClient;
use main_error::MainError;
use secretfile::{load, SecretError};
use std::time::Duration;
use thiserror::Error;
use time::OffsetDateTime;

mod config;
mod migrate;
mod store;

#[derive(Debug, Parser)]
struct Args {
    /// Config file
    config: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Request failed: {0}")]
    Request(#[from] std::io::Error),
    #[error(transparent)]
    Api(#[from] demostf_client::Error),
    #[error("Backup timed out")]
    Timeout,
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Secret(#[from] SecretError),
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let config = Config::load(args.config)?;

    let store = Store::new(config.storage.root, &config.storage.base_url);
    let api_key = load(&config.api.key_file)?;
    let backend = config.migrate.to_backend;
    let age = config.migrate.age;
    let migrate = Migrator::new(
        store,
        ApiClient::with_base_url(config.api.url).expect("invalid base url"),
        backend,
        api_key,
    );
    migrate
        .migrate_till(
            &config.migrate.from_backend,
            OffsetDateTime::now_utc() - Duration::from_secs(age),
        )
        .await?;
    Ok(())
}
