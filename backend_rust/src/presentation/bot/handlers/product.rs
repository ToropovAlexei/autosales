use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use shared_dtos::list_response::ListResponse;

use crate::{
    errors::api::{ApiResult, ErrorResponse},
    middlewares::bot_auth::AuthBot,
    models::product::ProductListQuery,
    presentation::admin::dtos::product::ProductResponse,
    services::product::ProductServiceTrait,
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_products))
        .route("/{id}", get(get_product))
}

#[utoipa::path(
    get,
    path = "/api/bot/products",
    tag = "Products",
    responses(
        (status = 200, description = "Products list", body = ListResponse<ProductResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn list_products(
    State(state): State<Arc<AppState>>,
    _bot: AuthBot,
    query: ProductListQuery,
) -> ApiResult<Json<ListResponse<ProductResponse>>> {
    let products = state.product_service.get_list(query).await?;

    Ok(Json(ListResponse {
        total: products.total,
        items: products
            .items
            .into_iter()
            .map(ProductResponse::from)
            .collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/bot/products/{id}",
    tag = "Products",
    responses(
        (status = 200, description = "Product details", body = ProductResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn get_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<ProductResponse>> {
    let product = state.product_service.get_by_id(id).await?;

    Ok(Json(ProductResponse::from(product)))
}
