use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use shared_dtos::{error::ApiErrorResponse, list_response::ListResponse};

use crate::{
    errors::api::ApiResult, middlewares::bot_auth::AuthBot,
    presentation::admin::dtos::category::CategoryAdminResponse,
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
        (status = 200, description = "List of categories", body = ListResponse<CategoryAdminResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn list_categories(
    State(state): State<Arc<AppState>>,
    _bot: AuthBot,
) -> ApiResult<Json<ListResponse<CategoryAdminResponse>>> {
    let categories = state.category_service.get_list().await?;

    Ok(Json(ListResponse {
        total: categories.len() as i64,
        items: categories
            .into_iter()
            .map(CategoryAdminResponse::from)
            .collect(),
    }))
}
