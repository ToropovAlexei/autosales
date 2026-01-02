use std::sync::Arc;

use axum::Router;

use crate::{presentation::images::handlers::image, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().nest("/images", image::router())
}
