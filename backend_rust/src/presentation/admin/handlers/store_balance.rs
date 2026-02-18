use rust_decimal::{
    Decimal,
    prelude::{FromPrimitive, ToPrimitive},
};
use shared_dtos::{
    balance_request::CreateStoreBalanceRequestAdminRequest, error::ApiErrorResponse,
    store_balance::StoreBalanceAdminResponse,
};
use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::{
    errors::api::{ApiError, ApiResult},
    middlewares::{
        context::RequestContext,
        require_permission::{RequirePermission, StoreBalanceRead, StoreBalanceWithdraw},
        validator::ValidatedJson,
    },
    services::{
        auth::AuthUser,
        store_balance_request::{
            CreateStoreBalanceRequestCommand, StoreBalanceRequestServiceTrait,
        },
        transaction::TransactionServiceTrait,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route(
        "/",
        get(get_store_balance).post(create_store_balance_request),
    )
}

#[utoipa::path(
    get,
    path = "/api/admin/store-balance",
    tag = "Store balance",
    responses(
        (status = 200, description = "Store balance", body = StoreBalanceAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn get_store_balance(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<StoreBalanceRead>,
) -> ApiResult<Json<StoreBalanceAdminResponse>> {
    let last_transaction = state.transaction_service.get_last().await;

    match last_transaction {
        Err(e) => {
            if matches!(e, ApiError::NotFound(_)) {
                return Ok(Json(StoreBalanceAdminResponse { balance: 0.0 }));
            }
            Err(e)
        }
        Ok(last_transaction) => Ok(Json(StoreBalanceAdminResponse {
            balance: last_transaction
                .store_balance_after
                .to_f64()
                .unwrap_or_default(),
        })),
    }
}

#[utoipa::path(
    post,
    path = "/api/admin/store-balance",
    tag = "Store balance",
    responses(
        (status = 200, description = "Store balance request created"),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn create_store_balance_request(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    // TODO make one permission to manage store balance
    _perm: RequirePermission<StoreBalanceWithdraw>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<CreateStoreBalanceRequestAdminRequest>,
) -> ApiResult<()> {
    let _ = state
        .store_balance_request_service
        .create(CreateStoreBalanceRequestCommand {
            admin_user_id: user.id,
            amount_rub: Decimal::from_f64(payload.amount_rub)
                .ok_or_else(|| ApiError::BadRequest("Invalid amount".into()))?,
            request_type: payload.request_type,
            wallet_address: payload.wallet_address,
            ctx,
        })
        .await?;

    Ok(())
}
