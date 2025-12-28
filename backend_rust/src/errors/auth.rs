use thiserror::Error;

use crate::errors::repository::RepositoryError;

pub type AuthResult<T> = Result<T, AuthError>;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("missing token")]
    MissingToken,
    #[error("invalid token")]
    InvalidToken,
    #[error("token revoked")]
    TokenRevoked,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("invalid 2FA code")]
    Invalid2FACode,
    #[error("{0}")]
    InternalServerError(String),
}

impl From<RepositoryError> for AuthError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(_msg) => AuthError::InvalidCredentials,
            _ => AuthError::InternalServerError(err.to_string()),
        }
    }
}
