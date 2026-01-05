use std::sync::Arc;

use axum::Router;

use crate::{
    presentation::admin::handlers::{
        admin_user, audit_log, auth, bot, category, customer, image, me, permission, product, role,
        settings, stock_movement, transaction,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/categories", category::router())
        .nest("/me", me::router())
        .nest("/admin-users", admin_user::router())
        .nest("/roles", role::router())
        .nest("/permissions", permission::router())
        .nest("/transactions", transaction::router())
        .nest("/products", product::router())
        .nest("/images", image::router())
        .nest("/stock-movements", stock_movement::router())
        .nest("/customers", customer::router())
        .nest("/settings", settings::router())
        .nest("/audit-logs", audit_log::router())
        .nest("/bots", bot::router())
}
