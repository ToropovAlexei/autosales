use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{patch, post},
};
use bigdecimal::{BigDecimal, FromPrimitive, Zero};

use crate::{
    errors::api::{ApiError, ApiResult},
    middlewares::{
        context::RequestContext,
        require_permission::{BotsCreate, BotsRead, BotsUpdate, RequirePermission},
        validator::ValidatedJson,
    },
    models::bot::{BotListQuery, BotType},
    presentation::admin::dtos::{
        bot::{BotResponse, NewBotRequest, UpdateBotRequest},
        list_response::ListResponse,
    },
    services::{
        auth::AuthUser,
        bot::{BotServiceTrait, CreateBotCommand, UpdateBotCommand},
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_bot).get(list_bots))
        .route("/{id}", patch(update_bot))
}

#[utoipa::path(
    post,
    path = "/api/admin/bots",
    tag = "Bots",
    request_body = NewBotRequest,
    responses(
        (status = 200, description = "Bot created", body = BotResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn create_bot(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    _perm: RequirePermission<BotsCreate>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<NewBotRequest>,
) -> ApiResult<Json<BotResponse>> {
    let bot = state
        .bot_service
        .create(
            CreateBotCommand {
                created_by: Some(user.id),
                is_active: true,
                is_primary: false,
                owner_id: None,
                referral_percentage: BigDecimal::zero(),
                token: payload.token,
                r#type: BotType::Main,
            },
            ctx,
        )
        .await?;

    Ok(Json(bot.into()))
}

#[utoipa::path(
    get,
    path = "/api/admin/bots",
    tag = "Bots",
    responses(
        (status = 200, description = "Bot list", body = ListResponse<BotResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn list_bots(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<BotsRead>,
    query: BotListQuery,
) -> ApiResult<Json<ListResponse<BotResponse>>> {
    let bots = state.bot_service.get_list(query).await?;

    Ok(Json(ListResponse {
        total: bots.total,
        items: bots.items.into_iter().map(BotResponse::from).collect(),
    }))
}

#[utoipa::path(
    patch,
    path = "/api/admin/bots/{id}",
    tag = "Bots",
    request_body = UpdateBotRequest,
    responses(
        (status = 200, description = "Bot updated", body = BotResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn update_bot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    user: AuthUser,
    _perm: RequirePermission<BotsUpdate>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<UpdateBotRequest>,
) -> ApiResult<Json<BotResponse>> {
    let bot = state
        .bot_service
        .update(
            UpdateBotCommand {
                id,
                is_active: payload.is_active,
                is_primary: payload.is_primary,
                referral_percentage: payload
                    .referral_percentage
                    .map(|p| {
                        BigDecimal::from_f64(p).ok_or_else(|| {
                            ApiError::BadRequest("Invalid referral percentage".into())
                        })
                    })
                    .transpose()?,
                updated_by: Some(user.id),
                username: None,
            },
            ctx,
        )
        .await?;

    Ok(Json(bot.into()))
}
