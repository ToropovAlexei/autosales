use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::{
    errors::api::ApiResult, middlewares::bot_auth::AuthBot,
    presentation::bot::dtos::settings::SettingsResponse, services::settings::SettingsServiceTrait,
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
        (status = 200, description = "Bot settings", body = SettingsResponse),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_settings(
    State(state): State<Arc<AppState>>,
    _bot: AuthBot,
) -> ApiResult<Json<SettingsResponse>> {
    let settings = state.settings_service.load_settings().await?;
    Ok(Json(SettingsResponse::from(settings)))
}
