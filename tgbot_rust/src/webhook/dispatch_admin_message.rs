use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};
use shared_dtos::notification::DispatchAdminMessage;

use crate::{
    AppState,
    errors::{AppError, AppResult},
    manager_bot::dispatch_admin_message,
};

pub async fn dispatch_admin_message_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    payload: Json<DispatchAdminMessage>,
) -> AppResult<impl IntoResponse> {
    let service_key = headers
        .get("X-API-KEY")
        .ok_or_else(|| AppError::AuthenticationError("Missing X-API-KEY".to_string()))?
        .to_str()
        .map_err(|_| AppError::AuthenticationError("Invalid header value".to_string()))?;

    if service_key != state.config.service_api_key {
        return Err(AppError::AuthenticationError("Invalid API key".to_string()));
    }

    dispatch_admin_message(state.config.clone(), state.api.clone(), payload.0).await?;

    Ok(())
}
