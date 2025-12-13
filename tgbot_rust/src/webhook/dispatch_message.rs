use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};
use deadpool_redis::redis::AsyncCommands;

use crate::{
    AppState,
    errors::{AppError, AppResult},
    models::DispatchMessagePayload,
};

#[axum::debug_handler]
pub async fn dispatch_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    payload: Json<DispatchMessagePayload>,
) -> AppResult<impl IntoResponse> {
    let service_key = headers
        .get("X-API-KEY")
        .ok_or_else(|| AppError::AuthenticationError("Missing X-API-KEY".to_string()))?
        .to_str()
        .map_err(|_| AppError::AuthenticationError("Invalid header value".to_string()))?;

    if service_key != state.config.service_token {
        return Err(AppError::AuthenticationError("Invalid API key".to_string()));
    }

    let channel = format!("bot-notifications:{}", payload.bot_name);
    let message_json = serde_json::to_string(&payload.0).map_err(|err| {
        AppError::InternalServerError(format!("Failed to serialize message: {err}"))
    })?;

    let mut conn = state.redis_pool.get().await?;

    conn.publish::<String, String, ()>(channel.clone(), message_json)
        .await
        .map_err(|err| AppError::InternalServerError(format!("Redis publish error: {err}")))?;

    Ok(())
}
