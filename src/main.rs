use main_error::MainError;
use thiserror::Error;

mod migrate;
mod store;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Request failed: {0}")]
    Request(#[from] std::io::Error),
    #[error(transparent)]
    Api(#[from] demostf_client::Error),
}

fn main() -> Result<(), MainError> {
    println!("Hello, world!");
    Ok(())
}
