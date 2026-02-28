use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};
use deadpool_redis::redis::AsyncCommands;
use shared_dtos::notification::DispatchMessagePayload;
use std::time::Instant;

use crate::{
    AppState,
    errors::{AppError, AppResult},
};

pub async fn dispatch_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    payload: Json<DispatchMessagePayload>,
) -> AppResult<impl IntoResponse> {
    let started_at = Instant::now();
    let service_key = headers
        .get("X-API-KEY")
        .ok_or_else(|| AppError::AuthenticationError("Missing X-API-KEY".to_string()))?
        .to_str()
        .map_err(|_| AppError::AuthenticationError("Invalid header value".to_string()))?;

    if service_key != state.config.service_api_key {
        return Err(AppError::AuthenticationError("Invalid API key".to_string()));
    }

    let channel = format!("bot-notifications:{}", payload.bot_id);
    let message_json = serde_json::to_string(&payload.0).map_err(|err| {
        AppError::InternalServerError(format!("Failed to serialize message: {err}"))
    })?;

    let mut conn = state.redis_pool.get().await?;

    let publish_started = Instant::now();
    conn.publish::<String, String, ()>(channel.clone(), message_json)
        .await
        .map_err(|err| AppError::InternalServerError(format!("Redis publish error: {err}")))?;
    let publish_elapsed_ms = publish_started.elapsed().as_millis();
    let total_elapsed_ms = started_at.elapsed().as_millis();

    if total_elapsed_ms >= 500 {
        tracing::warn!(
            bot_id = payload.bot_id,
            telegram_id = payload.telegram_id,
            publish_elapsed_ms,
            total_elapsed_ms,
            "Slow dispatch-message webhook request"
        );
    } else {
        tracing::info!(
            bot_id = payload.bot_id,
            telegram_id = payload.telegram_id,
            publish_elapsed_ms,
            total_elapsed_ms,
            "dispatch-message webhook request handled"
        );
    }

    Ok(())
}
