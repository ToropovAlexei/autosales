use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};

use crate::{
    errors::api::ApiResult,
    middlewares::validator::ValidatedJson,
    presentation::admin::dtos::auth::{
        LoginStep1Request, LoginStep1Response, LoginStep2Request, LoginStep2Response,
    },
    services::auth::AuthUser,
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login_step1))
        .route("/login/2fa", post(login_step2))
        .route("/logout", post(logout))
}

#[utoipa::path(
    post,
    path = "/api/admin/auth/login",
    tag = "Auth",
    request_body = LoginStep1Request,
    responses(
        (status = 200, description = "Login successful", body = LoginStep1Response),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn login_step1(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<LoginStep1Request>,
) -> ApiResult<Json<LoginStep1Response>> {
    let temp_token = state
        .auth_service
        .login_step1(payload.login, payload.password)
        .await?;

    Ok(Json(LoginStep1Response {
        temp_token: temp_token.token,
    }))
}

#[utoipa::path(
    post,
    path = "/api/admin/auth/login/2fa",
    tag = "Auth",
    request_body = LoginStep2Request,
    responses(
        (status = 200, description = "Login successful", body = LoginStep2Response),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn login_step2(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<LoginStep2Request>,
) -> ApiResult<Json<LoginStep2Response>> {
    let access_token = state
        .auth_service
        .login_step2(&payload.temp_token, &payload.code)
        .await?;

    Ok(Json(LoginStep2Response {
        token: access_token.jti,
    }))
}

#[utoipa::path(
    post,
    path = "/api/admin/auth/logout",
    tag = "Auth",
    responses(
        (status = 200, description = "Logout successful"),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn logout(State(state): State<Arc<AppState>>, user: AuthUser) -> ApiResult<()> {
    state.auth_service.logout(user.token).await?;
    Ok(())
}
