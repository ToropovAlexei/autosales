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
