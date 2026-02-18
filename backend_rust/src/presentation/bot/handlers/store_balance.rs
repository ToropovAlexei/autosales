use shared_dtos::{
    balance_request::{CompleteStoreBalanceRequestBotRequest, RejectStoreBalanceRequestBotRequest},
    error::ApiErrorResponse,
};
use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    routing::post,
};

use crate::{
    errors::api::ApiResult,
    middlewares::{validator::ValidatedJson, verified_service::VerifiedService},
    services::store_balance_request::{
        CompleteStoreBalanceRequestCommand, RejectStoreBalanceRequestCommand,
        StoreBalanceRequestServiceTrait,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{id}/complete", post(complete_store_balance_request))
        .route("/{id}/reject", post(reject_store_balance_request))
}

#[utoipa::path(
    post,
    path = "/api/bot/store-balance/{id}/complete",
    tag = "Store balance",
    responses(
        (status = 200, description = "Store balance request approved"),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn complete_store_balance_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _service: VerifiedService,
    ValidatedJson(payload): ValidatedJson<CompleteStoreBalanceRequestBotRequest>,
) -> ApiResult<()> {
    let _ = state
        .store_balance_request_service
        .complete(CompleteStoreBalanceRequestCommand {
            id,
            tg_user_id: payload.tg_user_id,
        })
        .await?;

    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/bot/store-balance/{id}/reject",
    tag = "Store balance",
    responses(
        (status = 200, description = "Store balance request rejected"),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn reject_store_balance_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _service: VerifiedService,
    ValidatedJson(payload): ValidatedJson<RejectStoreBalanceRequestBotRequest>,
) -> ApiResult<()> {
    let _ = state
        .store_balance_request_service
        .reject(RejectStoreBalanceRequestCommand {
            id,
            tg_user_id: payload.tg_user_id,
        })
        .await?;

    Ok(())
}
