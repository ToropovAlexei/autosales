use std::sync::Arc;

use axum::Router;

use crate::{
    presentation::admin::handlers::{auth, category},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/category", category::router())
}
