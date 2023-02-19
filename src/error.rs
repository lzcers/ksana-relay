use crate::database::Error as DatabaseError;
use crate::nostr::Error as NostrError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RelayError {
    #[error("relay error: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("nostr error: {0}")]
    NostrError(#[from] NostrError),
}
