use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use validator::ValidationErrors;

// Re-export repository error
pub mod repo;
pub use repo::RepositoryError;

/// The primary error type for the service layer.
#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    Validation(String),
    Forbidden(String),
    Internal(anyhow::Error),
}

impl From<RepositoryError> for ServiceError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(msg) => ServiceError::NotFound(msg),
            _ => ServiceError::Internal(anyhow::anyhow!(err)),
        }
    }
}

impl From<anyhow::Error> for ServiceError {
    fn from(err: anyhow::Error) -> Self {
        ServiceError::Internal(err)
    }
}

impl From<ValidationErrors> for ServiceError {
    fn from(err: ValidationErrors) -> Self {
        // In a real app, you might want to format this better
        ServiceError::Validation(err.to_string())
    }
}

/// The primary error type for the API layer.
#[derive(Debug)]
pub struct ApiError(ServiceError);

// Implement conversion from ServiceError to ApiError
impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        ApiError(err)
    }
}

impl<T> From<T> for ApiError
where
    T: Into<ServiceError>,
{
    fn from(err: T) -> Self {
        ApiError(err.into())
    }
}

// Implement conversion from our custom error type to a response.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ServiceError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            ServiceError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ServiceError::Internal(ref e) => {
                // Log the internal error for debugging
                tracing::error!("Internal server error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected error occurred.".to_string(),
                )
            }
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}