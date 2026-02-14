use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use shared_dtos::error::ApiErrorResponse;

use crate::{
    errors::api::ApiResult,
    middlewares::{
        context::RequestContext,
        require_permission::{
            PricingEdit, PricingRead, RequirePermission, SettingsEdit, SettingsRead,
        },
        validator::ValidatedJson,
    },
    presentation::admin::dtos::settings::{
        BotSettingsAdminResponse, PricingSettingsAdminResponse, UpdateBotSettingsAdminRequest,
        UpdatePricingSettingsAdminRequest,
    },
    services::{
        auth::AuthUser,
        settings::{SettingsServiceTrait, UpdateSettingsCommand},
    },
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
        (status = 200, description = "Pricing settings", body = PricingSettingsAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn get_pricing_settings(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<PricingRead>,
) -> ApiResult<Json<PricingSettingsAdminResponse>> {
    let settings = state.settings_service.load_settings().await?;

    Ok(Json(PricingSettingsAdminResponse::from(settings)))
}

#[utoipa::path(
    patch,
    path = "/api/admin/settings/pricing",
    tag = "Settings",
    responses(
        (status = 200, description = "Pricing settings updated", body = PricingSettingsAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn update_pricing_settings(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    _perm: RequirePermission<PricingEdit>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<UpdatePricingSettingsAdminRequest>,
) -> ApiResult<Json<PricingSettingsAdminResponse>> {
    let mut command = UpdateSettingsCommand::from(payload);
    command.updated_by = user.id;
    let category = state.settings_service.update(command, ctx).await?;

    Ok(Json(category.into()))
}

#[utoipa::path(
    get,
    path = "/api/admin/settings/bot",
    tag = "Settings",
    responses(
        (status = 200, description = "Bot settings", body = BotSettingsAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn get_bot_settings(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<SettingsRead>,
) -> ApiResult<Json<BotSettingsAdminResponse>> {
    let settings = state.settings_service.load_settings().await?;

    Ok(Json(BotSettingsAdminResponse::from(settings)))
}

#[utoipa::path(
    patch,
    path = "/api/admin/settings/bot",
    tag = "Settings",
    responses(
        (status = 200, description = "Bot settings updated", body = BotSettingsAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn update_bot_settings(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    _perm: RequirePermission<SettingsEdit>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<UpdateBotSettingsAdminRequest>,
) -> ApiResult<Json<BotSettingsAdminResponse>> {
    let mut command = UpdateSettingsCommand::from(payload);
    command.updated_by = user.id;
    let category = state.settings_service.update(command, ctx).await?;

    Ok(Json(category.into()))
}
