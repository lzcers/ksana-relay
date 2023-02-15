use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("database connect faild: {0}")]
    DatabaseConnectionError(#[from] sqlx::Error),

    #[error("database serde faild: {0}")]
    DatabaseSerdeJsonFaild(#[from] serde_json::Error),
}
