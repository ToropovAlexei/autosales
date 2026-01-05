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

use std::sync::Arc;

use axum::{Router, http, routing::get};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

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
        .merge(presentation::images::router::router())
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new())
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .with_state(app_state)
}

pub fn init_tracing() {
    let filter = EnvFilter::new("info")
        .add_directive("sqlx::query=info".parse().unwrap())
        .add_directive("tower_http::trace=debug".parse().unwrap());

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(filter)
        .init();
}

/// Run database migrations
pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    tracing::info!("Running database migrations");
    sqlx::migrate!("./migrations").run(pool).await
}
