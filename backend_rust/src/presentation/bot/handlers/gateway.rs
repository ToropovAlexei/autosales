use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::{
    errors::api::ApiResult,
    middlewares::verified_service::VerifiedService,
    models::payment::PaymentSystem,
    presentation::{admin::dtos::list_response::ListResponse, bot::dtos::gateway::GatewayResponse},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(get_gateways))
}

#[utoipa::path(
    get,
    path = "/api/bot/gateways",
    tag = "Bots",
    responses(
        (status = 200, description = "Gateways", body = ListResponse<GatewayResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_gateways(
    State(_state): State<Arc<AppState>>,
    _service: VerifiedService,
) -> ApiResult<Json<ListResponse<GatewayResponse>>> {
    let mut items = vec![
        GatewayResponse {
            name: PaymentSystem::PlatformCard,
            display_name: "Платформа (карта)".to_string(),
        },
        GatewayResponse {
            name: PaymentSystem::PlatformSBP,
            display_name: "Платформа (СБП)".to_string(),
        },
    ];

    #[cfg(feature = "mock-payments-provider")]
    items.push(GatewayResponse {
        name: PaymentSystem::Mock,
        display_name: "Криптоплатежи (мок-провайдер)".to_string(),
    });

    Ok(Json(ListResponse {
        total: items.len() as i64,
        items,
    }))
}
