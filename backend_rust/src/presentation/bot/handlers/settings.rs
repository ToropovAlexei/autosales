use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use shared_dtos::{
    error::ApiErrorResponse,
    settings::{SettingsBotResponse, UpdateBotManagedSettingsBotRequest},
};

use crate::{
    errors::api::ApiResult,
    middlewares::{validator::ValidatedJson, verified_service::VerifiedService},
    services::settings::{SettingsServiceTrait, UpdateSettingsCommand},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(get_settings).patch(update_bot_managed_settings))
}

#[utoipa::path(
    get,
    path = "/api/bot/settings",
    tag = "Bot",
    responses(
        (status = 200, description = "Bot settings", body = SettingsBotResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn get_settings(
    State(state): State<Arc<AppState>>,
    _service: VerifiedService,
) -> ApiResult<Json<SettingsBotResponse>> {
    let settings = state.settings_service.load_settings().await?;
    Ok(Json(SettingsBotResponse::from(settings)))
}

#[utoipa::path(
    patch,
    path = "/api/bot/settings",
    tag = "Bot",
    responses(
        (status = 200, description = "Bot-managed settings updated", body = SettingsBotResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn update_bot_managed_settings(
    State(state): State<Arc<AppState>>,
    _service: VerifiedService,
    ValidatedJson(payload): ValidatedJson<UpdateBotManagedSettingsBotRequest>,
) -> ApiResult<Json<SettingsBotResponse>> {
    let settings = state
        .settings_service
        .update(UpdateSettingsCommand {
            updated_by: 1,
            ..UpdateSettingsCommand::from(payload)
        })
        .await?;

    Ok(Json(SettingsBotResponse::from(settings)))
}
