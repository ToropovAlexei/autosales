use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use shared_dtos::list_response::ListResponse;

use crate::{
    errors::api::ApiResult, middlewares::bot_auth::AuthBot,
    presentation::admin::dtos::category::CategoryResponse,
    services::category::CategoryServiceTrait, state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(list_categories))
}

#[utoipa::path(
    get,
    path = "/api/bot/categories",
    tag = "Categories",
    responses(
        (status = 200, description = "List of categories", body = ListResponse<CategoryResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn list_categories(
    State(state): State<Arc<AppState>>,
    _bot: AuthBot,
) -> ApiResult<Json<ListResponse<CategoryResponse>>> {
    let categories = state.category_service.get_list().await?;

    Ok(Json(ListResponse {
        total: categories.len() as i64,
        items: categories.into_iter().map(CategoryResponse::from).collect(),
    }))
}
