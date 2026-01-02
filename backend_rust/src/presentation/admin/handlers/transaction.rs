use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::{
    errors::api::ApiResult,
    middlewares::require_permission::{RequirePermission, TransactionsRead},
    models::common::ListQuery,
    presentation::admin::dtos::{list_response::ListResponse, transaction::TransactionResponse},
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
        (status = 200, description = "Transactions list", body = ListResponse<TransactionResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn list_transactions(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<TransactionsRead>,
    query: ListQuery,
) -> ApiResult<Json<ListResponse<TransactionResponse>>> {
    let transactions = state.transaction_service.get_list(query).await?;

    Ok(Json(ListResponse {
        total: transactions.total,
        items: transactions
            .items
            .into_iter()
            .map(TransactionResponse::from)
            .collect(),
    }))
}
