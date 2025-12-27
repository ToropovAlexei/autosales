use axum::http;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    middlewares::validator::ValidatedJson,
    presentation::dtos::category::{CategoryResponse, NewCategoryRequest, UpdateCategoryRequest},
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
    ValidatedJson(payload): ValidatedJson<NewCategoryRequest>,
) -> Result<Json<CategoryResponse>, http::StatusCode> {
    //
    // TODO: get user from context
    let created_by = 1;
    let category = state
        .category_service
        .create(
            payload.name,
            payload.parent_id,
            payload.image_id,
            created_by,
        )
        .await
        .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(category.into()))
}

async fn list_categories(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CategoryResponse>>, http::StatusCode> {
    let categories = state
        .category_service
        .get_all()
        .await
        .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let categories_dto = categories.into_iter().map(CategoryResponse::from).collect();

    Ok(Json(categories_dto))
}

async fn get_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<CategoryResponse>, http::StatusCode> {
    let category = state
        .category_service
        .get_by_id(id)
        .await
        .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

    match category {
        Some(category) => Ok(Json(category.into())),
        None => Err(http::StatusCode::NOT_FOUND),
    }
}

async fn update_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    ValidatedJson(payload): ValidatedJson<UpdateCategoryRequest>,
) -> Result<Json<CategoryResponse>, http::StatusCode> {
    // TODO: get user from context
    let created_by = 1;

    let category = state
        .category_service
        .update(
            id,
            payload.name,
            payload.parent_id,
            payload.image_id,
            created_by,
        )
        .await
        .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(category.into()))
}

async fn delete_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<(), http::StatusCode> {
    state
        .category_service
        .delete(id)
        .await
        .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}
