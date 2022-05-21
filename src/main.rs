use crate::migrate::Migrator;
use crate::store::Store;
use demostf_client::ApiClient;
use main_error::MainError;
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use time::OffsetDateTime;

mod migrate;
mod store;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Request failed: {0}")]
    Request(#[from] std::io::Error),
    #[error(transparent)]
    Api(#[from] demostf_client::Error),
    #[error("Backup timed out")]
    Timeout,
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    tracing_subscriber::fmt::init();

    let mut args: HashMap<_, _> = dotenv::vars().collect();
    let store = Store::new(
        args.get("STORAGE_ROOT").expect("no STORAGE_ROOT set"),
        args.get("BASE_URL").expect("no BASE_URL set"),
    );
    let api_key = args.remove("KEY").expect("no KEY set");
    let backend = args.remove("BACKEND").expect("no BACKEND set");
    let age: u64 = args
        .get("AGE")
        .expect("no AGE set")
        .parse()
        .expect("invalid AGE");
    let migrate = Migrator::new(store, ApiClient::new(), backend, api_key);
    migrate
        .migrate_till(
            "static",
            OffsetDateTime::now_utc() - Duration::from_secs(age),
        )
        .await?;
    Ok(())
}
