pub mod bin;
pub mod config;
pub mod db;
pub mod errors;
pub mod infrastructure;
pub mod middlewares;
pub mod models;
pub mod presentation;
pub mod services;
pub mod state;
pub mod workers;

use std::sync::Arc;

use axum::{Router, http, routing::get};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnFailure, TraceLayer},
};
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, time::LocalTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::state::AppState;

pub async fn healthz() -> &'static str {
    "healthy"
}

pub fn create_app(app_state: Arc<AppState>) -> Router {
    use axum::http::Method;
    use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};

    let cors = CorsLayer::new()
        .allow_origin(
            app_state
                .config
                .cors_origins
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true);

    Router::new()
        .route("/healthz", get(healthz))
        .nest("/api/admin", presentation::admin::router::router())
        .nest("/api/bot", presentation::bot::router::router())
        .nest("/api/webhook", presentation::webhook::router::router())
        .nest("/api", presentation::images::router::router())
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new())
                .on_response(|response: &axum::response::Response, latency: std::time::Duration, span: &tracing::Span| {
                    let status = response.status();
                    let latency_ms = latency.as_millis();

                    if status.is_server_error() {
                        tracing::error!(parent: span, http.status = %status, latency_ms, "request failed");
                    } else if status.is_client_error() {
                        tracing::warn!(parent: span, http.status = %status, latency_ms, "request rejected");
                    } else {
                        tracing::info!(parent: span, http.status = %status, latency_ms, "request completed");
                    }
                })
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
        .with_state(app_state)
}

pub fn init_tracing() {
    let filter = EnvFilter::new("info")
        .add_directive("sqlx::query=info".parse().unwrap())
        .add_directive("tower_http::trace=debug".parse().unwrap());

    let time_format = LocalTime::rfc_3339();

    let console_layer = fmt::layer()
        .with_timer(time_format.clone())
        .with_writer(std::io::stdout)
        .with_target(false)
        .with_level(true)
        .pretty();

    let _ = std::fs::create_dir_all("logs");
    let file_appender = rolling::daily("logs", "app.log");
    let file_layer = fmt::layer()
        .json()
        .with_timer(time_format)
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_level(true);

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .with(filter)
        .init();
}

/// Run database migrations
pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    tracing::info!("Running database migrations");
    sqlx::migrate!("./migrations").run(pool).await
}
