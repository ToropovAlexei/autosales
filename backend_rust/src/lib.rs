use std::sync::Arc;

use axum::{Router, http::HeaderValue, routing::get};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::state::AppState;

pub mod config;
pub mod db;
pub mod state;

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
                .parse::<HeaderValue>()
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

    Router::new().route("/healthz", get(healthz)).layer(cors)
}

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .init();
}

/// Run database migrations
pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    tracing::info!("Running database migrations");
    sqlx::migrate!("./migrations").run(pool).await
}
