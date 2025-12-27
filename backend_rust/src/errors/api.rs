use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;
use validator::ValidationErrors;

use crate::errors::repository::RepositoryError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationErrors),
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error")]
    InternalServerError,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize, ToSchema)]
pub struct ValidationErrorResponse {
    errors: std::collections::HashMap<String, String>,
}

impl From<RepositoryError> for ApiError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(msg) => ApiError::NotFound(msg),
            RepositoryError::ForeignKeyViolation(msg) => ApiError::BadRequest(msg),
            RepositoryError::UniqueViolation(msg) => ApiError::BadRequest(msg),
            RepositoryError::Validation(msg) => ApiError::BadRequest(msg),
            RepositoryError::OptimisticLockViolation => ApiError::InternalServerError,
            RepositoryError::QueryFailed(_err) => ApiError::InternalServerError,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!("Error occurred: {}", self.to_string());

        match self {
            ApiError::ValidationError(err) => {
                let mut errors = std::collections::HashMap::new();
                for (field, field_errors) in err.field_errors() {
                    let messages: String = field_errors.iter().fold(String::new(), |mut acc, e| {
                        if !acc.is_empty() {
                            acc.push_str(", ");
                        }
                        let msg = e
                            .message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "Invalid value".to_string());
                        acc.push_str(&msg);
                        acc
                    });
                    errors.insert(field.to_string(), messages);
                }
                (
                    StatusCode::BAD_REQUEST,
                    Json(ValidationErrorResponse { errors }),
                )
                    .into_response()
            }
            _ => {
                let (status, message) = match self {
                    ApiError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
                    ApiError::AuthorizationError(msg) => (StatusCode::FORBIDDEN, msg),
                    ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
                    ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
                    ApiError::InternalServerError => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                    ),
                    ApiError::ValidationError(_) => unreachable!(),
                };

                let body = Json(ErrorResponse { error: message });

                (status, body).into_response()
            }
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        ApiError::BadRequest(rejection.to_string())
    }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;
