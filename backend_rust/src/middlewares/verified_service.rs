use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{errors::api::ApiError, state::AppState};

pub struct VerifiedService;

impl FromRequestParts<Arc<AppState>> for VerifiedService {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("X-API-KEY")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::AuthenticationError(
                "Missing auth header".to_string(),
            ))?;

        if auth_header != state.config.service_api_key {
            return Err(ApiError::AuthenticationError(
                "Invalid auth header".to_string(),
            ));
        }

        Ok(VerifiedService {})
    }
}
