use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use shared_dtos::{
    bot::{BotBotResponse, NewBotBotRequest, UpdateBotBotRequest},
    error::ApiErrorResponse,
    list_response::ListResponse,
};

use crate::{
    errors::api::ApiResult,
    middlewares::{bot_auth::AuthBot, validator::ValidatedJson, verified_service::VerifiedService},
    models::bot::{BotListQuery, BotType},
    services::{
        bot::{BotServiceTrait, CreateBotCommand, UpdateBotCommand},
        customer::CustomerServiceTrait,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_bot).get(list_bots))
        .route("/primary", get(get_primary_bots))
        .route("/{id}", get(get_bot).patch(update_bot).delete(delete_bot))
}

#[utoipa::path(
    get,
    path = "/api/bot/bots/{id}",
    tag = "Bots",
    responses(
        (status = 200, description = "Bot found", body = BotBotResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn get_bot(
    State(state): State<Arc<AppState>>,
    _bot: AuthBot,
    Path(id): Path<i64>,
) -> ApiResult<Json<BotBotResponse>> {
    let bot = state.bot_service.get_by_id(id).await?;
    Ok(Json(bot.into()))
}

#[utoipa::path(
    post,
    path = "/api/bot/bots",
    tag = "Bots",
    request_body = NewBotBotRequest,
    responses(
        (status = 200, description = "Bot created", body = BotBotResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn create_bot(
    State(state): State<Arc<AppState>>,
    _bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<NewBotBotRequest>,
) -> ApiResult<Json<BotBotResponse>> {
    let owner = state
        .customer_service
        .get_by_telegram_id(payload.owner_id)
        .await?;
    let bot = state
        .bot_service
        .create(CreateBotCommand {
            token: payload.token,
            is_active: true,
            is_primary: false,
            r#type: BotType::Referral,
            created_by: None,
            owner_id: Some(owner.id),
            ctx: None,
        })
        .await?;

    Ok(Json(bot.into()))
}

#[utoipa::path(
    get,
    path = "/api/bot/bots",
    tag = "Bots",
    responses(
        (status = 200, description = "Bot list", body = ListResponse<BotBotResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn list_bots(
    State(state): State<Arc<AppState>>,
    _service: VerifiedService,
    query: BotListQuery,
) -> ApiResult<Json<ListResponse<BotBotResponse>>> {
    let bots = state.bot_service.get_list(query).await?;

    Ok(Json(ListResponse {
        total: bots.total,
        items: bots.items.into_iter().map(BotBotResponse::from).collect(),
    }))
}

#[utoipa::path(
    patch,
    path = "/api/bot/bots/{id}",
    tag = "Bots",
    request_body = UpdateBotBotRequest,
    responses(
        (status = 200, description = "Bot updated", body = BotBotResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn update_bot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<UpdateBotBotRequest>,
) -> ApiResult<Json<BotBotResponse>> {
    let bot = state
        .bot_service
        .update(UpdateBotCommand {
            id,
            is_active: payload.is_active,
            is_primary: payload.is_primary,
            updated_by: None,
            referral_percentage: None,
            username: None,
            ctx: None,
        })
        .await?;

    Ok(Json(bot.into()))
}

#[utoipa::path(
    get,
    path = "/api/bot/bots/primary",
    tag = "Bots",
    responses(
        (status = 200, description = "Bot list", body = ListResponse<BotBotResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn get_primary_bots(
    State(state): State<Arc<AppState>>,
    _service: VerifiedService,
) -> ApiResult<Json<ListResponse<BotBotResponse>>> {
    let bots = state.bot_service.get_primary_bots().await?;

    Ok(Json(ListResponse {
        total: bots.len() as i64,
        items: bots.into_iter().map(BotBotResponse::from).collect(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/bot/bots/{id}",
    tag = "Bots",
    responses(
        (status = 200, description = "Bot deleted"),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn delete_bot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<()> {
    state.bot_service.delete(id).await?;
    Ok(())
}
