use std::sync::Arc;

use axum::Router;

use crate::{
    presentation::bot::handlers::{
        bot, can_operate, captcha, category, customer, gateway, invoice, order, product, settings,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/settings", settings::router())
        .nest("/categories", category::router())
        .nest("/products", product::router())
        .nest("/bots", bot::router())
        .nest("/can-operate", can_operate::router())
        .nest("/captcha", captcha::router())
        .nest("/customers", customer::router())
        .nest("/gateways", gateway::router())
        .nest("/invoices", invoice::router())
        .nest("/orders", order::router())
}
