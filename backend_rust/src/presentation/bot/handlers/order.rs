use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};

use crate::{
    errors::api::ApiResult,
    middlewares::{bot_auth::AuthBot, validator::ValidatedJson},
    presentation::bot::dtos::order::{PurchaseRequest, PurchaseResponse},
    services::purchase::{PurchaseProductCommand, PurchaseServiceTrait},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", post(purchase))
}

#[utoipa::path(
    get,
    path = "/api/bot/orders",
    tag = "Orders",
    responses(
        (status = 200, description = "Create order", body = PurchaseResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn purchase(
    State(state): State<Arc<AppState>>,
    bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<PurchaseRequest>,
) -> ApiResult<Json<PurchaseResponse>> {
    let result = state
        .purchase_service
        .purchase_product(PurchaseProductCommand {
            amount: 1,
            bot_id: bot.bot_id,
            product_id: payload.product_id,
            telegram_id: payload.telegram_id,
        })
        .await?;

    Ok(Json(PurchaseResponse {
        product_name: result.product_name,
        balance: result.balance,
        details: result.details,
        fulfilled_text: result.fulfilled_text,
        fulfilled_image_id: result.fulfilled_image_id,
        price: result.price,
    }))
}
