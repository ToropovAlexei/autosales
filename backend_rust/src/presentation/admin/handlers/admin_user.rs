use axum::http::StatusCode;
use shared_dtos::{error::ApiErrorResponse, list_response::ListResponse};
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
            AdminUsersCreate, AdminUsersDelete, AdminUsersRead, AdminUsersUpdate, RbacManage,
            RequirePermission,
        },
        validator::ValidatedJson,
    },
    models::user_permission::{UpdateUserPermissions, UpsertUserPermission},
    presentation::admin::dtos::{
        admin_user::{
            AdminUserAdminResponse, AdminUserWithRolesAdminResponse, NewAdminUserAdminRequest,
            NewAdminUserAdminResponse, UpdateAdminUserAdminRequest,
        },
        permission::PermissionAdminResponse,
        user_permission::{UpdateUserPermissionsRequest, UserPermissionAdminResponse},
    },
    services::{
        admin_user::{
            AdminUserServiceTrait, CreateAdminUser, DeleteAdminUserCommand, UpdateAdminUserCommand,
        },
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
    request_body = NewAdminUserAdminRequest,
    responses(
        (status = 200, description = "Admin user created", body = AdminUserAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn create_admin_user(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    _perm: RequirePermission<AdminUsersCreate>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<NewAdminUserAdminRequest>,
) -> ApiResult<Json<NewAdminUserAdminResponse>> {
    let admin_user = state
        .admin_user_service
        .create(
            CreateAdminUser {
                login: payload.login,
                password: payload.password,
                created_by: user.id,
                roles: payload.roles,
            },
            ctx,
        )
        .await?;

    Ok(Json(NewAdminUserAdminResponse {
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
        (status = 200, description = "Admin users list", body = ListResponse<AdminUserWithRolesAdminResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn list_admin_users(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<AdminUsersRead>,
) -> ApiResult<Json<ListResponse<AdminUserWithRolesAdminResponse>>> {
    let admin_users = state.admin_user_service.get_all_users_with_roles().await?;

    Ok(Json(ListResponse {
        total: admin_users.len() as i64,
        items: admin_users
            .into_iter()
            .map(AdminUserWithRolesAdminResponse::from)
            .collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/admin-users/{id}",
    tag = "Admin Users",
    responses(
        (status = 200, description = "Admin user details", body = AdminUserAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn get_admin_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<AdminUsersRead>,
) -> ApiResult<Json<AdminUserAdminResponse>> {
    let admin_user = state.admin_user_service.get_by_id(id).await?;

    Ok(Json(admin_user.into()))
}

#[utoipa::path(
    patch,
    path = "/api/admin/admin-users/{id}",
    tag = "Admin Users",
    request_body = UpdateAdminUserAdminRequest,
    responses(
        (status = 200, description = "Admin user updated", body = AdminUserAdminResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn update_admin_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    user: AuthUser,
    _perm: RequirePermission<AdminUsersUpdate>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<UpdateAdminUserAdminRequest>,
) -> ApiResult<Json<AdminUserAdminResponse>> {
    let admin_user = state
        .admin_user_service
        .update(
            id,
            UpdateAdminUserCommand {
                login: payload.login,
                password: payload.password,
                telegram_id: payload.telegram_id,
                roles: payload.roles,
                updated_by: user.id,
            },
            ctx,
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
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn delete_admin_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    user: AuthUser,
    _perm: RequirePermission<AdminUsersDelete>,
    ctx: RequestContext,
) -> ApiResult<StatusCode> {
    state
        .admin_user_service
        .delete(
            DeleteAdminUserCommand {
                id,
                deleted_by: user.id,
            },
            ctx,
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/admin/admin-users/{id}/permissions",
    tag = "Admin Users",
    responses(
        (status = 200, description = "Admin user permissions", body = ListResponse<PermissionAdminResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn get_admin_user_permissions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<RbacManage>,
) -> ApiResult<Json<ListResponse<UserPermissionAdminResponse>>> {
    let permissions = state.permission_service.get_for_admin_user(id).await?;

    Ok(Json(ListResponse {
        total: permissions.len() as i64,
        items: permissions
            .into_iter()
            .map(|p| UserPermissionAdminResponse {
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
        (status = 200, description = "Admin user permissions updated", body = ListResponse<PermissionAdminResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn update_admin_user_permissions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    user: AuthUser,
    _perm: RequirePermission<RbacManage>,
    ValidatedJson(payload): ValidatedJson<UpdateUserPermissionsRequest>,
) -> ApiResult<Json<ListResponse<UserPermissionAdminResponse>>> {
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
            .map(|p| UserPermissionAdminResponse {
                effect: p.effect,
                id: p.permission_id,
            })
            .collect(),
    }))
}
