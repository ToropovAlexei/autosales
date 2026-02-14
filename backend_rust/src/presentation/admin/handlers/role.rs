use axum::{
    http::StatusCode,
    routing::{get, patch},
};
use shared_dtos::list_response::ListResponse;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::post,
};

use crate::{
    errors::api::{ApiResult, ErrorResponse},
    middlewares::{
        require_permission::{RbacManage, RequirePermission},
        validator::ValidatedJson,
    },
    models::{
        role::{NewRole, UpdateRole},
        role_permission::UpdateRolePermissions,
    },
    presentation::admin::dtos::{
        permission::PermissionResponse,
        role::{NewRoleRequest, RoleResponse, UpdateRoleRequest},
        role_permission::UpdateRolePermissionsRequest,
    },
    services::{
        auth::AuthUser, permission::PermissionServiceTrait, role::RoleServiceTrait,
        role_permission::RolePermissionServiceTrait,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_role).get(list_roles))
        .route("/{id}", patch(update_role).delete(delete_role))
        .route(
            "/{id}/permissions",
            get(get_role_permissions).patch(update_role_permissions),
        )
}

#[utoipa::path(
    post,
    path = "/api/admin/roles",
    tag = "Roles",
    responses(
        (status = 200, description = "Role created", body = RoleResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn create_role(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    _perm: RequirePermission<RbacManage>,
    ValidatedJson(payload): ValidatedJson<NewRoleRequest>,
) -> ApiResult<Json<RoleResponse>> {
    let role = state
        .role_service
        .create(NewRole {
            description: payload.description,
            name: payload.name,
            created_by: user.id,
        })
        .await?;

    Ok(Json(role.into()))
}

#[utoipa::path(
    get,
    path = "/api/admin/roles",
    tag = "Roles",
    responses(
        (status = 200, description = "Admin user roles", body = ListResponse<RoleResponse>),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn list_roles(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<RbacManage>,
) -> ApiResult<Json<ListResponse<RoleResponse>>> {
    let roles = state.role_service.get_list().await?;

    Ok(Json(ListResponse {
        total: roles.len() as i64,
        items: roles.into_iter().map(RoleResponse::from).collect(),
    }))
}

#[utoipa::path(
    patch,
    path = "/api/admin/roles/{id}",
    tag = "Roles",
    responses(
        (status = 200, description = "Role updated", body = RoleResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn update_role(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<RbacManage>,
    ValidatedJson(payload): ValidatedJson<UpdateRoleRequest>,
) -> ApiResult<Json<RoleResponse>> {
    let role = state
        .role_service
        .update(
            id,
            UpdateRole {
                description: payload.description,
                name: payload.name,
            },
        )
        .await?;

    Ok(Json(role.into()))
}

#[utoipa::path(
    delete,
    path = "/api/admin/roles/{id}",
    tag = "Roles",
    responses(
        (status = 204, description = "Role deleted"),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn delete_role(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<RbacManage>,
) -> ApiResult<StatusCode> {
    state.role_service.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/admin/roles/{id}/permissions",
    tag = "Roles",
    responses(
        (status = 200, description = "Role permissions", body = ListResponse<PermissionResponse>),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn get_role_permissions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<RbacManage>,
) -> ApiResult<Json<ListResponse<PermissionResponse>>> {
    let permissions = state.permission_service.get_for_role(id).await?;

    Ok(Json(ListResponse {
        total: permissions.len() as i64,
        items: permissions
            .into_iter()
            .map(PermissionResponse::from)
            .collect(),
    }))
}

#[utoipa::path(
    patch,
    path = "/api/admin/roles/{id}/permissions",
    tag = "Roles",
    responses(
        (status = 200, description = "Role permissions updated", body = ListResponse<PermissionResponse>),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn update_role_permissions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    user: AuthUser,
    _perm: RequirePermission<RbacManage>,
    ValidatedJson(payload): ValidatedJson<UpdateRolePermissionsRequest>,
) -> ApiResult<Json<ListResponse<PermissionResponse>>> {
    state
        .role_permission_service
        .update_role_permissions(UpdateRolePermissions {
            added: payload.added,
            removed: payload.removed,
            created_by: user.id,
            role_id: id,
        })
        .await?;

    let permissions = state.permission_service.get_for_role(id).await?;

    Ok(Json(ListResponse {
        total: permissions.len() as i64,
        items: permissions
            .into_iter()
            .map(PermissionResponse::from)
            .collect(),
    }))
}
