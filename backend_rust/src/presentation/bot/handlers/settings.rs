use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use shared_dtos::settings::SettingsBotResponse;

use crate::{
    errors::api::{ApiResult, ErrorResponse},
    middlewares::bot_auth::AuthBot,
    services::settings::SettingsServiceTrait,
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(get_settings))
}

#[utoipa::path(
    get,
    path = "/api/bot/settings",
    tag = "Bot",
    responses(
        (status = 200, description = "Bot settings", body = SettingsBotResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn get_settings(
    State(state): State<Arc<AppState>>,
    _bot: AuthBot,
) -> ApiResult<Json<SettingsBotResponse>> {
    let settings = state.settings_service.load_settings().await?;
    Ok(Json(SettingsBotResponse::from(settings)))
}
