use axum::{
    http::StatusCode,
    routing::{get, patch},
};
use shared_dtos::{
    error::ApiErrorResponse,
    list_response::ListResponse,
    permission::PermissionAdminResponse,
    role::{NewRoleAdminRequest, RoleAdminResponse, UpdateRoleAdminRequest},
    role_permission::UpdateRolePermissionsAdminRequest,
};
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::post,
};

use crate::{
    errors::api::ApiResult,
    middlewares::{
        require_permission::{RbacManage, RequirePermission},
        validator::ValidatedJson,
    },
    models::{
        role::{NewRole, UpdateRole},
        role_permission::UpdateRolePermissions,
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
        (status = 200, description = "Role created", body = RoleAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn create_role(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    _perm: RequirePermission<RbacManage>,
    ValidatedJson(payload): ValidatedJson<NewRoleAdminRequest>,
) -> ApiResult<Json<RoleAdminResponse>> {
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
        (status = 200, description = "Admin user roles", body = ListResponse<RoleAdminResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn list_roles(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<RbacManage>,
) -> ApiResult<Json<ListResponse<RoleAdminResponse>>> {
    let roles = state.role_service.get_list().await?;

    Ok(Json(ListResponse {
        total: roles.len() as i64,
        items: roles.into_iter().map(RoleAdminResponse::from).collect(),
    }))
}

#[utoipa::path(
    patch,
    path = "/api/admin/roles/{id}",
    tag = "Roles",
    responses(
        (status = 200, description = "Role updated", body = RoleAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn update_role(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<RbacManage>,
    ValidatedJson(payload): ValidatedJson<UpdateRoleAdminRequest>,
) -> ApiResult<Json<RoleAdminResponse>> {
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
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
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
        (status = 200, description = "Role permissions", body = ListResponse<PermissionAdminResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn get_role_permissions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<RbacManage>,
) -> ApiResult<Json<ListResponse<PermissionAdminResponse>>> {
    let permissions = state.permission_service.get_for_role(id).await?;

    Ok(Json(ListResponse {
        total: permissions.len() as i64,
        items: permissions
            .into_iter()
            .map(PermissionAdminResponse::from)
            .collect(),
    }))
}

#[utoipa::path(
    patch,
    path = "/api/admin/roles/{id}/permissions",
    tag = "Roles",
    responses(
        (status = 200, description = "Role permissions updated", body = ListResponse<PermissionAdminResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn update_role_permissions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    user: AuthUser,
    _perm: RequirePermission<RbacManage>,
    ValidatedJson(payload): ValidatedJson<UpdateRolePermissionsAdminRequest>,
) -> ApiResult<Json<ListResponse<PermissionAdminResponse>>> {
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
            .map(PermissionAdminResponse::from)
            .collect(),
    }))
}
