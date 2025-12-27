use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{errors::api::ApiError, services::auth::AuthUser, state::AppState};

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::AuthenticationError(
                "Missing auth header".to_string(),
            ))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::AuthenticationError(
                "Invalid auth header".to_string(),
            ))?;

        state
            .auth_service
            .authenticate(token)
            .await
            .map_err(|e| ApiError::AuthenticationError(e.to_string()))
    }
}
