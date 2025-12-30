use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::{
    errors::api::ApiResult,
    middlewares::require_permission::{RbacManage, RequirePermission},
    presentation::admin::dtos::{list_response::ListResponse, permission::PermissionResponse},
    services::{auth::AuthUser, permission::PermissionServiceTrait},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(list_permissions))
}

#[utoipa::path(
    get,
    path = "/api/admin/permissions",
    tag = "Permissions",
    responses(
        (status = 200, description = "Admin user permissions", body = ListResponse<PermissionResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn list_permissions(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<RbacManage>,
) -> ApiResult<Json<ListResponse<PermissionResponse>>> {
    let permissions = state.permission_service.get_list().await?;

    Ok(Json(ListResponse {
        total: permissions.len() as i64,
        items: permissions
            .into_iter()
            .map(PermissionResponse::from)
            .collect(),
    }))
}
