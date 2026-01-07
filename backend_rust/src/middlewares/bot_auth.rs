use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{
    errors::api::ApiError, middlewares::verified_service::VerifiedService,
    services::bot::BotServiceTrait, state::AppState,
};

pub struct AuthBot {
    pub bot_id: i64,
}

impl FromRequestParts<Arc<AppState>> for AuthBot {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        VerifiedService::from_request_parts(parts, state).await?;

        let bot_id = parts
            .headers
            .get("X-BOT-ID")
            .and_then(|v| v.to_str().ok())
            .and_then(|id| id.parse::<i64>().ok())
            .ok_or(ApiError::AuthenticationError("Missing bot id".to_string()))?;

        if state.bot_service.get_by_id(bot_id).await.is_err() {
            return Err(ApiError::AuthenticationError("Invalid bot id".to_string()));
        };

        Ok(AuthBot { bot_id })
    }
}
