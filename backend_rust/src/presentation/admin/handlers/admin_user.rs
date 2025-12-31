use axum::http::StatusCode;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

use crate::{
    errors::api::ApiResult,
    middlewares::{
        require_permission::{
            AdminUsersCreate, AdminUsersDelete, AdminUsersRead, AdminUsersUpdate, RbacManage,
            RequirePermission,
        },
        validator::ValidatedJson,
    },
    models::user_permission::{UpdateUserPermissions, UpsertUserPermission},
    presentation::admin::dtos::{
        admin_user::{
            AdminUserResponse, AdminUserWithRolesResponse, NewAdminUserRequest,
            NewAdminUserResponse, UpdateAdminUserRequest,
        },
        list_response::ListResponse,
        permission::PermissionResponse,
        user_permission::{UpdateUserPermissionsRequest, UserPermissionResponse},
    },
    services::{
        admin_user::{AdminUserServiceTrait, CreateAdminUser, UpdateAdminUserCommand},
        auth::AuthUser,
        permission::PermissionServiceTrait,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_admin_user).get(list_admin_users))
        .route(
            "/{id}",
            get(get_admin_user)
                .patch(update_admin_user)
                .delete(delete_admin_user),
        )
        .route(
            "/{id}/permissions",
            get(get_admin_user_permissions).patch(update_admin_user_permissions),
        )
}

#[utoipa::path(
    post,
    path = "/api/admin/admin-users",
    tag = "Admin Users",
    request_body = NewAdminUserRequest,
    responses(
        (status = 200, description = "Admin user created", body = AdminUserResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn create_admin_user(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    _perm: RequirePermission<AdminUsersCreate>,
    ValidatedJson(payload): ValidatedJson<NewAdminUserRequest>,
) -> ApiResult<Json<NewAdminUserResponse>> {
    let admin_user = state
        .admin_user_service
        .create(CreateAdminUser {
            login: payload.login,
            password: payload.password,
            created_by: user.id,
            roles: payload.roles,
        })
        .await?;

    Ok(Json(NewAdminUserResponse {
        id: admin_user.id,
        created_at: admin_user.created_at,
        created_by: admin_user.created_by,
        deleted_at: admin_user.deleted_at,
        login: admin_user.login,
        telegram_id: admin_user.telegram_id,
        two_fa_qr_code: admin_user.two_fa_qr_code,
        two_fa_secret: admin_user.two_fa_secret,
        updated_at: admin_user.updated_at,
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/admin-users",
    tag = "Admin Users",
    responses(
        (status = 200, description = "Admin users list", body = ListResponse<AdminUserWithRolesResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn list_admin_users(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<AdminUsersRead>,
) -> ApiResult<Json<ListResponse<AdminUserWithRolesResponse>>> {
    let admin_users = state.admin_user_service.get_all_users_with_roles().await?;

    Ok(Json(ListResponse {
        total: admin_users.len() as i64,
        items: admin_users
            .into_iter()
            .map(AdminUserWithRolesResponse::from)
            .collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/admin-users/{id}",
    tag = "Admin Users",
    responses(
        (status = 200, description = "Admin user details", body = AdminUserResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_admin_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<AdminUsersRead>,
) -> ApiResult<Json<AdminUserResponse>> {
    let admin_user = state.admin_user_service.get_by_id(id).await?;

    Ok(Json(admin_user.into()))
}

#[utoipa::path(
    patch,
    path = "/api/admin/admin-users/{id}",
    tag = "Admin Users",
    request_body = UpdateAdminUserRequest,
    responses(
        (status = 200, description = "Admin user updated", body = AdminUserResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn update_admin_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<AdminUsersUpdate>,
    ValidatedJson(payload): ValidatedJson<UpdateAdminUserRequest>,
) -> ApiResult<Json<AdminUserResponse>> {
    let admin_user = state
        .admin_user_service
        .update(
            id,
            UpdateAdminUserCommand {
                login: payload.login,
                password: payload.password,
                telegram_id: payload.telegram_id,
                roles: payload.roles,
            },
        )
        .await?;

    Ok(Json(admin_user.into()))
}

#[utoipa::path(
    delete,
    path = "/api/admin/admin-users/{id}",
    tag = "Admin Users",
    responses(
        (status = 204, description = "Admin user deleted"),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn delete_admin_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<AdminUsersDelete>,
) -> ApiResult<StatusCode> {
    state.admin_user_service.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/admin/admin-users/{id}/permissions",
    tag = "Admin Users",
    responses(
        (status = 200, description = "Admin user permissions", body = ListResponse<PermissionResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_admin_user_permissions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<RbacManage>,
) -> ApiResult<Json<ListResponse<UserPermissionResponse>>> {
    let permissions = state.permission_service.get_for_admin_user(id).await?;

    Ok(Json(ListResponse {
        total: permissions.len() as i64,
        items: permissions
            .into_iter()
            .map(|p| UserPermissionResponse {
                effect: p.effect,
                id: p.permission_id,
            })
            .collect(),
    }))
}

#[utoipa::path(
    patch,
    path = "/api/admin/admin-users/{id}/permissions",
    tag = "Admin Users",
    request_body = UpdateUserPermissionsRequest,
    responses(
        (status = 200, description = "Admin user permissions updated", body = ListResponse<PermissionResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn update_admin_user_permissions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    user: AuthUser,
    _perm: RequirePermission<RbacManage>,
    ValidatedJson(payload): ValidatedJson<UpdateUserPermissionsRequest>,
) -> ApiResult<Json<ListResponse<UserPermissionResponse>>> {
    state
        .permission_service
        .update_admin_user_permissions(UpdateUserPermissions {
            user_id: id,
            created_by: user.id,
            removed: payload.removed,
            upserted: payload
                .upserted
                .iter()
                .map(|x| UpsertUserPermission {
                    id: x.id,
                    effect: x.effect,
                })
                .collect(),
        })
        .await?;

    let permissions = state.permission_service.get_for_admin_user(id).await?;

    Ok(Json(ListResponse {
        total: permissions.len() as i64,
        items: permissions
            .into_iter()
            .map(|p| UserPermissionResponse {
                effect: p.effect,
                id: p.permission_id,
            })
            .collect(),
    }))
}
