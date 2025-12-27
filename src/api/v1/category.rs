use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use validator::Validate;

use crate::{
    api::dto::category::{CreateCategoryDto, UpdateCategoryDto},
    errors::ApiError,
    repositories::category::{NewCategory, UpdateCategory},
    state::AppState,
};

// The handler functions are now defined before they are used.

async fn create_category(
    State(state): State<Arc<AppState>>,
    // In a real app, you would get the user from the auth middleware
    // user: AuthenticatedUser,
    Json(dto): Json<CreateCategoryDto>,
) -> Result<impl IntoResponse, ApiError> {
    dto.validate()?;

    let new_category_input = NewCategory {
        name: dto.name,
        parent_id: dto.parent_id,
        image_id: dto.image_id,
        created_by: 1, // Replace with actual user ID, e.g., user.id
    };

    let created_category = state.category_service.create(new_category_input).await?;

    Ok((StatusCode::CREATED, Json(created_category)))
}

async fn list_categories(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let categories = state.category_service.list_all().await?;
    Ok(Json(categories))
}

async fn get_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    let category = state.category_service.get_by_id(id).await?;
    Ok(Json(category))
}

async fn update_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateCategoryDto>,
) -> Result<impl IntoResponse, ApiError> {
    dto.validate()?;

    let update_category_input = UpdateCategory {
        name: dto.name,
        parent_id: dto.parent_id,
        image_id: dto.image_id,
        position: dto.position,
        is_active: dto.is_active,
    };

    let updated_category = state
        .category_service
        .update(id, update_category_input)
        .await?;

    Ok(Json(updated_category))
}

async fn delete_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    state.category_service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn category_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_category).get(list_categories))
        .route(
            "/:id",
            get(get_category)
                .patch(update_category)
                .delete(delete_category),
        )
}