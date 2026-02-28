use axum::{Router, routing::post};
use tracing::Level;

mod dispatch_admin_message;
mod dispatch_message;
use crate::AppState;
use dispatch_admin_message::dispatch_admin_message_handler;
use dispatch_message::dispatch_message;
use tower_http::trace::TraceLayer;

pub fn create_webhook_service(app_state: AppState) -> Router {
    Router::new()
        .route("/webhook/dispatch-message", post(dispatch_message))
        .route(
            "/webhook/dispatch-admin-message",
            post(dispatch_admin_message_handler),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(tower_http::trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(app_state)
}
