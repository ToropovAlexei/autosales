use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use shared_dtos::{error::ApiErrorResponse, list_response::ListResponse};

use crate::{
    errors::api::ApiResult,
    middlewares::require_permission::{RequirePermission, TransactionsRead},
    models::transaction::TransactionListQuery,
    presentation::admin::dtos::transaction::TransactionAdminResponse,
    services::{auth::AuthUser, transaction::TransactionServiceTrait},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(list_transactions))
}

#[utoipa::path(
    get,
    path = "/api/admin/transactions",
    tag = "Transactions",
    responses(
        (status = 200, description = "Transactions list", body = ListResponse<TransactionAdminResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn list_transactions(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<TransactionsRead>,
    query: TransactionListQuery,
) -> ApiResult<Json<ListResponse<TransactionAdminResponse>>> {
    let transactions = state.transaction_service.get_list(query).await?;

    Ok(Json(ListResponse {
        total: transactions.total,
        items: transactions
            .items
            .into_iter()
            .map(TransactionAdminResponse::from)
            .collect(),
    }))
}
