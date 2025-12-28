use thiserror::Error;

pub type TotpEncryptorResult<T> = Result<T, TotpEncryptorError>;

#[derive(Debug, Error)]
pub enum TotpEncryptorError {
    #[error("missing secret")]
    MissingSecret,
    #[error("Decode error: {0}")]
    DecodeError(String),
}
