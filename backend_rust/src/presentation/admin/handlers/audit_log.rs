use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use shared_dtos::list_response::ListResponse;

use crate::{
    errors::api::{ApiResult, ErrorResponse},
    middlewares::require_permission::{AuditLogRead, RequirePermission},
    models::audit_log::AuditLogListQuery,
    presentation::admin::dtos::audit_log::AuditLogResponse,
    services::{audit_log::AuditLogServiceTrait, auth::AuthUser},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(list_audit_logs))
}

#[utoipa::path(
    get,
    path = "/api/admin/audit-logs",
    tag = "Audit Logs",
    responses(
        (status = 200, description = "List of audit logs", body = ListResponse<AuditLogResponse>),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn list_audit_logs(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<AuditLogRead>,
    query: AuditLogListQuery,
) -> ApiResult<Json<ListResponse<AuditLogResponse>>> {
    let audit_logs = state.audit_logs_service.get_list(query).await?;

    Ok(Json(ListResponse {
        total: audit_logs.total,
        items: audit_logs
            .items
            .into_iter()
            .map(AuditLogResponse::from)
            .collect(),
    }))
}
