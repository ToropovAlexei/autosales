use std::sync::Arc;

use axum::{Json, Router, debug_handler, extract::State, routing::post};
use shared_dtos::error::ApiErrorResponse;

use crate::{
    errors::api::{ApiError, ApiResult},
    middlewares::{context::RequestContext, validator::ValidatedJson},
    models::audit_log::{AuditAction, AuditStatus, NewAuditLog},
    presentation::admin::dtos::auth::{
        LoginStep1Request, LoginStep1Response, LoginStep2Request, LoginStep2Response,
    },
    services::{
        audit_log::AuditLogServiceTrait,
        auth::{AuthServiceTrait, AuthUser},
    },
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
    security(()),
    request_body = LoginStep1Request,
    responses(
        (status = 200, description = "Login successful", body = LoginStep1Response),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
#[debug_handler]
async fn login_step1(
    State(state): State<Arc<AppState>>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<LoginStep1Request>,
) -> ApiResult<Json<LoginStep1Response>> {
    let temp_token = {
        match state
            .auth_service
            .login_step1(&payload.login, &payload.password)
            .await
        {
            Ok(temp_token) => temp_token,
            Err(e) => {
                state
                    .audit_logs_service
                    .create(NewAuditLog {
                        action: AuditAction::UserLogin,
                        status: AuditStatus::Failed,
                        admin_user_id: None,
                        customer_id: None,
                        ip_address: ctx.ip_address,
                        user_agent: ctx.user_agent,
                        request_id: Some(ctx.request_id),
                        error_message: Some(e.to_string()),
                        new_values: None,
                        old_values: None,
                        target_id: payload.login,
                        target_table: "temp_tokens".to_string(),
                    })
                    .await?;
                return Err(ApiError::from(e));
            }
        }
    };

    state
        .audit_logs_service
        .create(NewAuditLog {
            action: AuditAction::UserLogin,
            status: AuditStatus::Success,
            admin_user_id: Some(temp_token.user_id),
            customer_id: None,
            ip_address: ctx.ip_address,
            user_agent: ctx.user_agent,
            request_id: Some(ctx.request_id),
            error_message: None,
            new_values: None,
            old_values: None,
            target_id: temp_token.user_id.to_string(),
            target_table: "temp_tokens".to_string(),
        })
        .await?;

    Ok(Json(LoginStep1Response {
        temp_token: temp_token.token,
    }))
}

#[utoipa::path(
    post,
    path = "/api/admin/auth/login/2fa",
    tag = "Auth",
    security(()),
    request_body = LoginStep2Request,
    responses(
        (status = 200, description = "Login successful", body = LoginStep2Response),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
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
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn logout(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    ctx: RequestContext,
) -> ApiResult<()> {
    state.auth_service.logout(user.token).await?;
    state
        .audit_logs_service
        .create(NewAuditLog {
            action: AuditAction::UserLogout,
            status: AuditStatus::Success,
            admin_user_id: Some(user.id),
            customer_id: None,
            ip_address: ctx.ip_address,
            user_agent: ctx.user_agent,
            request_id: Some(ctx.request_id),
            error_message: None,
            new_values: None,
            old_values: None,
            target_id: user.id.to_string(),
            target_table: "active_tokens".to_string(),
        })
        .await?;
    Ok(())
}
