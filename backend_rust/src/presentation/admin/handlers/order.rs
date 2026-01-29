use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use shared_dtos::list_response::ListResponse;

use crate::{
    errors::api::ApiResult,
    middlewares::require_permission::{OrdersRead, RequirePermission},
    models::order::OrderListQuery,
    presentation::admin::dtos::order::OrderResponse,
    services::{auth::AuthUser, order::OrderServiceTrait},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(list_orders))
}

#[utoipa::path(
    get,
    path = "/api/admin/orders",
    tag = "Orders",
    responses(
        (status = 200, description = "List of orders", body = ListResponse<OrderResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn list_orders(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<OrdersRead>,
    query: OrderListQuery,
) -> ApiResult<Json<ListResponse<OrderResponse>>> {
    let orders = state.order_service.get_list(query).await?;

    Ok(Json(ListResponse {
        total: orders.total,
        items: orders.items.into_iter().map(OrderResponse::from).collect(),
    }))
}
