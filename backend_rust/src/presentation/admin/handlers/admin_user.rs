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
            AdminUsersCreate, AdminUsersDelete, AdminUsersRead, AdminUsersUpdate, RequirePermission,
        },
        validator::ValidatedJson,
    },
    presentation::admin::dtos::{
        admin_user::{
            AdminUserResponse, AdminUserWithRolesResponse, NewAdminUserRequest,
            UpdateAdminUserRequest,
        },
        list_response::ListResponse,
    },
    services::{
        admin_user::{AdminUserServiceTrait, CreateAdminUser, UpdateAdminUserCommand},
        auth::AuthUser,
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
) -> ApiResult<Json<AdminUserResponse>> {
    let admin_user = state
        .admin_user_service
        .create(CreateAdminUser {
            login: payload.login,
            password: payload.password,
            created_by: user.id,
        })
        .await?;

    Ok(Json(admin_user.into()))
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
