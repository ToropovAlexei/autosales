use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use shared_dtos::{error::ApiErrorResponse, list_response::ListResponse};

use crate::{
    errors::api::ApiResult,
    middlewares::require_permission::{RbacManage, RequirePermission},
    presentation::admin::dtos::permission::PermissionAdminResponse,
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
        (status = 200, description = "Admin user permissions", body = ListResponse<PermissionAdminResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn list_permissions(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<RbacManage>,
) -> ApiResult<Json<ListResponse<PermissionAdminResponse>>> {
    let permissions = state.permission_service.get_list().await?;

    Ok(Json(ListResponse {
        total: permissions.len() as i64,
        items: permissions
            .into_iter()
            .map(PermissionAdminResponse::from)
            .collect(),
    }))
}
