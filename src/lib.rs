// Temporary comment to force re-evaluation
use std::sync::Arc;

use axum::{
    http::{header, HeaderValue, Method},
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{api::v1, state::AppState};

pub mod api;
pub mod config;
pub mod db;
pub mod errors;
pub mod models;
pub mod repositories;
pub mod services;
pub mod state;

pub async fn healthz() -> &'static str {
    "healthy"
}

pub fn create_app(app_state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            app_state
                .config
                .cors_origins
                .parse::<HeaderValue>()
                .expect("Invalid CORS origin"),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
        .allow_credentials(true);

    let api_v1_routes = Router::new().nest("/categories", v1::category::category_routes());

    Router::new()
        .route("/healthz", get(healthz))
        .nest("/api/v1", api_v1_routes)
        .with_state(app_state)
        .layer(cors)
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
