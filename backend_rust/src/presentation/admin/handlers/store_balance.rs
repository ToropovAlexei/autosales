use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::{
    errors::api::{ApiError, ApiResult},
    middlewares::require_permission::{RequirePermission, StoreBalanceRead},
    presentation::admin::dtos::store_balance::StoreBalanceResponse,
    services::{auth::AuthUser, transaction::TransactionServiceTrait},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(get_store_balance))
}

#[utoipa::path(
    get,
    path = "/api/admin/store-balance",
    tag = "Store balance",
    responses(
        (status = 200, description = "Store balance", body = StoreBalanceResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_store_balance(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<StoreBalanceRead>,
) -> ApiResult<Json<StoreBalanceResponse>> {
    let last_transaction = state.transaction_service.get_last().await;

    match last_transaction {
        Err(e) => {
            if matches!(e, ApiError::NotFound(_)) {
                return Ok(Json(StoreBalanceResponse { balance: 0.0 }));
            }
            Err(e)
        }
        Ok(last_transaction) => Ok(Json(StoreBalanceResponse {
            balance: last_transaction
                .store_balance_after
                .to_f64()
                .unwrap_or_default(),
        })),
    }
}
