use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiClientError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Unsuccessful request: {0}")]
    Unsuccessful(String),
}

pub type ApiClientResult<T> = std::result::Result<T, ApiClientError>;
