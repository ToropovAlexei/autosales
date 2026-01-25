use std::sync::Arc;

use axum::{Json, Router, debug_handler, extract::State, routing::get};

use crate::{
    errors::api::ApiResult,
    presentation::admin::dtos::{admin_user::AdminUserResponse, list_response::ListResponse},
    services::{
        admin_user::AdminUserServiceTrait,
        auth::{AuthServiceTrait, AuthUser},
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_me))
        .route("/permissions", get(get_me_permissions))
}

#[utoipa::path(
    get,
    path = "/api/admin/me",
    tag = "Me",
    responses(
        (status = 200, description = "Admin user details", body = AdminUserResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_me(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> ApiResult<Json<AdminUserResponse>> {
    let admin_user = state.admin_user_service.get_by_id(user.id).await?;
    Ok(Json(AdminUserResponse::from(admin_user)))
}

#[utoipa::path(
    get,
    path = "/api/admin/me/permissions",
    tag = "Me",
    responses(
        (status = 200, description = "Admin user permissions", body = ListResponse<String>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
#[debug_handler]
async fn get_me_permissions(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> ApiResult<Json<ListResponse<String>>> {
    let user_permissions = state.auth_service.get_user_permissions(user.id).await?;
    Ok(Json(ListResponse {
        total: user_permissions.len() as i64,
        items: user_permissions,
    }))
}
