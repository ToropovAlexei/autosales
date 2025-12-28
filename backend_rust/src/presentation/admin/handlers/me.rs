use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::{
    errors::api::ApiResult,
    presentation::admin::dtos::admin_user::AdminUserResponse,
    services::{admin_user::AdminUserServiceTrait, auth::AuthUser},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(get_me))
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
