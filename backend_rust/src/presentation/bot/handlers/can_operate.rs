use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use shared_dtos::can_operate::CanOperateBotResponse;

use crate::{
    errors::api::ApiResult, middlewares::verified_service::VerifiedService,
    services::bot::BotServiceTrait, state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(can_operate))
}

#[utoipa::path(
    get,
    path = "/api/bot/can-operate",
    tag = "Bots",
    responses(
        (status = 200, description = "Can operate", body = CanOperateBotResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn can_operate(
    State(state): State<Arc<AppState>>,
    _service: VerifiedService,
) -> ApiResult<Json<CanOperateBotResponse>> {
    let can_operate = state.bot_service.can_operate().await?;

    Ok(Json(CanOperateBotResponse { can_operate }))
}
