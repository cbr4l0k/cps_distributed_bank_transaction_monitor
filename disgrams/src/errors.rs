use thiserror::Error;

#[derive(Debug, Error)]
pub enum DisgramsError {
    #[error("invalid transaction type byte: {0}")]
    InvalidTransactionType(u8),

    #[error("encryption failed")]
    EncryptionFailed,

    #[error("decryption failed")]
    DecryptionFailed,

    #[error("invalid encryption key")]
    InvalidKey,

    #[error("packet length {0} does not match expected {1}")]
    InvalidPacketLength(usize, usize),
}

pub type Result<T> = std::result::Result<T, DisgramsError>;
