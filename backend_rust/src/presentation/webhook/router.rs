use std::sync::Arc;

use axum::Router;

use crate::{presentation::webhook::handlers::payment, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().nest("/payment", payment::router())
}
