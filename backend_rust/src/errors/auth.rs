use thiserror::Error;

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
    #[error("internal server error")]
    InternalServerError,
}
