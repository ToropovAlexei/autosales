use axum::{http::StatusCode, routing::patch};
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
    models::role::{NewRole, UpdateRole},
    presentation::admin::dtos::{
        list_response::ListResponse,
        role::{NewRoleRequest, RoleResponse, UpdateRoleRequest},
    },
    services::{auth::AuthUser, role::RoleServiceTrait},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_role).get(list_roles))
        .route("/{id}", patch(update_role).delete(delete_role))
}

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

async fn delete_role(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<RbacManage>,
) -> ApiResult<StatusCode> {
    state.role_service.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}
