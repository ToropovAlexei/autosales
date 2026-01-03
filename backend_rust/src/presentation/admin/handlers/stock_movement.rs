use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::{
    errors::api::ApiResult,
    middlewares::require_permission::{RequirePermission, StockRead},
    models::stock_movement::StockMovementListQuery,
    presentation::admin::dtos::{
        list_response::ListResponse, stock_movement::StockMovementResponse,
    },
    services::{auth::AuthUser, stock_movement::StockMovementServiceTrait},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(list_stock_movement))
}

#[utoipa::path(
    get,
    path = "/api/admin/stock_movements",
    tag = "Stock Movements",
    responses(
        (status = 200, description = "List of stock movements", body = Vec<StockMovementResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn list_stock_movement(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<StockRead>,
    query: StockMovementListQuery,
) -> ApiResult<Json<ListResponse<StockMovementResponse>>> {
    let stock_movements = state.stock_movement_service.get_list(query).await?;

    Ok(Json(ListResponse {
        total: stock_movements.total,
        items: stock_movements
            .items
            .into_iter()
            .map(StockMovementResponse::from)
            .collect(),
    }))
}
