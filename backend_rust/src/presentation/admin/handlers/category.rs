use axum::http::StatusCode;
use shared_dtos::{
    category::{CategoryAdminResponse, NewCategoryAdminRequest, UpdateCategoryAdminRequest},
    error::ApiErrorResponse,
    list_response::ListResponse,
};
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

use crate::{
    errors::api::ApiResult,
    middlewares::{
        context::RequestContext,
        require_permission::{
            CategoriesCreate, CategoriesDelete, CategoriesRead, CategoriesUpdate, RequirePermission,
        },
        validator::ValidatedJson,
    },
    services::{
        auth::AuthUser,
        category::{
            CategoryServiceTrait, CreateCategoryCommand, DeleteCategoryCommand,
            UpdateCategoryCommand,
        },
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_category).get(list_categories))
        .route(
            "/{id}",
            get(get_category)
                .patch(update_category)
                .delete(delete_category),
        )
}

#[utoipa::path(
    post,
    path = "/api/admin/categories",
    tag = "Categories",
    request_body = NewCategoryAdminRequest,
    responses(
        (status = 200, description = "Category created", body = CategoryAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn create_category(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    _perm: RequirePermission<CategoriesCreate>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<NewCategoryAdminRequest>,
) -> ApiResult<Json<CategoryAdminResponse>> {
    let category = state
        .category_service
        .create(CreateCategoryCommand {
            created_by: user.id,
            image_id: payload.image_id,
            name: payload.name,
            parent_id: payload.parent_id,
            ctx: Some(ctx),
        })
        .await?;

    Ok(Json(category.into()))
}

#[utoipa::path(
    get,
    path = "/api/admin/categories",
    tag = "Categories",
    responses(
        (status = 200, description = "List of categories", body = ListResponse<CategoryAdminResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn list_categories(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<CategoriesRead>,
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

#[utoipa::path(
    get,
    path = "/api/admin/categories/{id}",
    tag = "Categories",
    responses(
        (status = 200, description = "Category details", body = CategoryAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn get_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<CategoriesRead>,
) -> ApiResult<Json<CategoryAdminResponse>> {
    let category = state.category_service.get_by_id(id).await?;

    Ok(Json(CategoryAdminResponse::from(category)))
}

#[utoipa::path(
    patch,
    path = "/api/admin/categories/{id}",
    tag = "Categories",
    request_body = UpdateCategoryAdminRequest,
    responses(
        (status = 200, description = "Category updated", body = CategoryAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn update_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    user: AuthUser,
    _perm: RequirePermission<CategoriesUpdate>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<UpdateCategoryAdminRequest>,
) -> ApiResult<Json<CategoryAdminResponse>> {
    let category = state
        .category_service
        .update(
            UpdateCategoryCommand {
                id,
                image_id: payload.image_id,
                name: payload.name,
                parent_id: payload.parent_id,
                position: payload.position,
                updated_by: user.id,
            },
            ctx,
        )
        .await?;

    Ok(Json(category.into()))
}

#[utoipa::path(
    delete,
    path = "/api/admin/categories/{id}",
    tag = "Categories",
    responses(
        (status = 204, description = "Category deleted"),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn delete_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    user: AuthUser,
    ctx: RequestContext,
    _perm: RequirePermission<CategoriesDelete>,
) -> ApiResult<StatusCode> {
    state
        .category_service
        .delete(
            DeleteCategoryCommand {
                id,
                deleted_by: user.id,
            },
            ctx,
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
