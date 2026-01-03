use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::{
    errors::api::ApiResult,
    middlewares::{
        require_permission::{
            PricingEdit, PricingRead, RequirePermission, SettingsEdit, SettingsRead,
        },
        validator::ValidatedJson,
    },
    models::settings::UpdateSettings,
    presentation::admin::dtos::settings::{
        BotSettingsResponse, PricingSettingsResponse, UpdateBotSettingsRequest,
        UpdatePricingSettingsRequest,
    },
    services::{auth::AuthUser, settings::SettingsServiceTrait},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/pricing",
            get(get_pricing_settings).patch(update_pricing_settings),
        )
        .route("/bot", get(get_bot_settings).patch(update_bot_settings))
}

#[utoipa::path(
    get,
    path = "/api/admin/settings/pricing",
    tag = "Settings",
    responses(
        (status = 200, description = "Pricing settings", body = PricingSettingsResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_pricing_settings(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<PricingRead>,
) -> ApiResult<Json<PricingSettingsResponse>> {
    let settings = state.settings_service.load_settings().await?;

    Ok(Json(PricingSettingsResponse::from(settings)))
}

#[utoipa::path(
    patch,
    path = "/api/admin/settings/pricing",
    tag = "Settings",
    responses(
        (status = 200, description = "Pricing settings updated", body = PricingSettingsResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn update_pricing_settings(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<PricingEdit>,
    ValidatedJson(payload): ValidatedJson<UpdatePricingSettingsRequest>,
) -> ApiResult<Json<PricingSettingsResponse>> {
    let category = state
        .settings_service
        .update(UpdateSettings::from(payload))
        .await?;

    Ok(Json(category.into()))
}

#[utoipa::path(
    get,
    path = "/api/admin/settings/bot",
    tag = "Settings",
    responses(
        (status = 200, description = "Bot settings", body = BotSettingsResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_bot_settings(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<SettingsRead>,
) -> ApiResult<Json<BotSettingsResponse>> {
    let settings = state.settings_service.load_settings().await?;

    Ok(Json(BotSettingsResponse::from(settings)))
}

#[utoipa::path(
    patch,
    path = "/api/admin/settings/bot",
    tag = "Settings",
    responses(
        (status = 200, description = "Bot settings updated", body = BotSettingsResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn update_bot_settings(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<SettingsEdit>,
    ValidatedJson(payload): ValidatedJson<UpdateBotSettingsRequest>,
) -> ApiResult<Json<BotSettingsResponse>> {
    let category = state
        .settings_service
        .update(UpdateSettings::from(payload))
        .await?;

    Ok(Json(category.into()))
}
