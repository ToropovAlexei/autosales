use axum::http::StatusCode;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

use crate::{
    errors::api::ApiResult,
    middlewares::validator::ValidatedJson,
    models::category::{NewCategory, UpdateCategory},
    presentation::dtos::category::{CategoryResponse, NewCategoryRequest, UpdateCategoryRequest},
    services::{auth::AuthUser, category::CategoryServiceTrait},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/categories", post(create_category).get(list_categories))
        .route(
            "/categories/:id",
            get(get_category)
                .patch(update_category)
                .delete(delete_category),
        )
}

async fn create_category(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    ValidatedJson(payload): ValidatedJson<NewCategoryRequest>,
) -> ApiResult<Json<CategoryResponse>> {
    let category = state
        .category_service
        .create(NewCategory {
            created_by: user.id,
            image_id: payload.image_id,
            name: payload.name,
            parent_id: payload.parent_id,
        })
        .await?;

    Ok(Json(category.into()))
}

async fn list_categories(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<CategoryResponse>>> {
    let categories = state.category_service.get_list().await?;

    let categories_dto = categories.into_iter().map(CategoryResponse::from).collect();

    Ok(Json(categories_dto))
}

async fn get_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ApiResult<Json<CategoryResponse>> {
    let category = state.category_service.get_by_id(id).await?;

    Ok(Json(CategoryResponse::from(category)))
}

async fn update_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    ValidatedJson(payload): ValidatedJson<UpdateCategoryRequest>,
) -> ApiResult<Json<CategoryResponse>> {
    let category = state
        .category_service
        .update(
            id,
            UpdateCategory {
                image_id: payload.image_id,
                name: payload.name,
                parent_id: payload.parent_id,
                position: payload.position,
            },
        )
        .await?;

    Ok(Json(category.into()))
}

async fn delete_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    state.category_service.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}
