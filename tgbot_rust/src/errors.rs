use std::convert::Infallible;

use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use deadpool_redis::PoolError;
use serde::Serialize;
use teloxide::{RequestError, dispatching::dialogue::RedisStorageError};
use thiserror::Error;

use crate::api::api_errors::ApiClientError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Redis pool error: {0}")]
    RedisPoolError(#[from] PoolError),
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    #[error("Teloxide error: {0}")]
    RequestError(#[from] RequestError),
    #[error("Redis infallible storage error: {0}")]
    RedisInfallibleStorageError(#[from] RedisStorageError<Infallible>),
    #[error("Redis storage error: {0}")]
    RedisStorageError(#[from] RedisStorageError<serde_json::Error>),
    #[error("Url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("API client error: {0}")]
    ApiClient(#[from] ApiClientError),
    #[error("Other error: {0}")]
    OtherError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Captcha generation error: {0}")]
    CaptchaError(String),
    #[error("Base64 decode error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("Bot unauthorized: {0}")]
    BotUnauthorized(String),
    #[error("Bot healthcheck failed: {0}")]
    BotHealthcheckFailed(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
pub struct ValidationErrorResponse {
    errors: std::collections::HashMap<String, String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("Error occurred: {}", self.to_string());
        let (status, message) = match self {
            AppError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::RedisPoolError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::RedisError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::RequestError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::RedisStorageError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string())
            }
            AppError::UrlParseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::ReqwestError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::InvalidHeaderValue(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string())
            }
            AppError::OtherError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::ApiClient(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::RedisInfallibleStorageError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string())
            }
            AppError::IOError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::CaptchaError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::Base64DecodeError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string())
            }
            AppError::BotUnauthorized(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::BotHealthcheckFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse { error: message });

        (status, body).into_response()
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::BadRequest(rejection.to_string())
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
