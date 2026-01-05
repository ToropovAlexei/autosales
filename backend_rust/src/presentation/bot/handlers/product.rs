use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::{
    errors::api::ApiResult,
    middlewares::bot_auth::AuthBot,
    models::product::ProductListQuery,
    presentation::admin::dtos::{list_response::ListResponse, product::ProductResponse},
    services::product::ProductServiceTrait,
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(list_products))
}

#[utoipa::path(
    get,
    path = "/api/bot/products",
    tag = "Products",
    responses(
        (status = 200, description = "Products list", body = ListResponse<ProductResponse>),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
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
