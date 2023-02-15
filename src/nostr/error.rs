use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// Wrong length hex string
    #[allow(dead_code)]
    #[error("Wrong length hex string")]
    WrongLengthHexString,
    /// Hex string decoding error
    #[error("Hex Decode Error: {0}")]
    HexDecode(#[from] hex::FromHexError),
    /// Serialization error
    #[error("JSON (de)serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("ECDSA Signature error: {0}")]
    Signature(#[from] k256::ecdsa::Error),

    #[error("verifier error")]
    #[allow(dead_code)]
    VerifierError,

    #[error("event verify error")]
    HashMismatch,
}
