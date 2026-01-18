use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};

use crate::{
    errors::api::ApiResult,
    middlewares::{
        context::RequestContext,
        require_permission::{BroadcastCreate, RequirePermission},
        validator::ValidatedJson,
    },
    presentation::admin::dtos::broadcast::{BroadcastResponse, NewBroadcastRequest},
    services::{
        auth::AuthUser,
        broadcast::{BroadcastServiceTrait, CreateBroadcastCommand},
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", post(create_broadcast))
}

#[utoipa::path(
    post,
    path = "/api/admin/broadcasts",
    tag = "Broadcast",
    responses(
        (status = 200, description = "Broadcast scheduled", body = BroadcastResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn create_broadcast(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    _perm: RequirePermission<BroadcastCreate>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<NewBroadcastRequest>,
) -> ApiResult<Json<BroadcastResponse>> {
    let broadcast = state
        .broadcast_service
        .create(CreateBroadcastCommand {
            content_image_id: payload.content_image_id,
            content_text: payload.content_text,
            created_by: user.id,
            ctx: Some(ctx),
            filters: payload.filters,
            scheduled_for: payload.scheduled_for,
        })
        .await?;

    Ok(Json(BroadcastResponse::from(broadcast)))
}
