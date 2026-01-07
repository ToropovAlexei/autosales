use std::sync::Arc;

use axum::Router;

use crate::{
    presentation::bot::handlers::{bot, category, product, settings},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/settings", settings::router())
        .nest("/categories", category::router())
        .nest("/products", product::router())
        .nest("/bots", bot::router())
}
