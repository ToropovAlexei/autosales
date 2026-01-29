use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use shared_dtos::invoice::{GatewayBotResponse, PaymentSystem};

use crate::{
    errors::api::ApiResult, middlewares::verified_service::VerifiedService,
    presentation::admin::dtos::list_response::ListResponse, state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(get_gateways))
}

#[utoipa::path(
    get,
    path = "/api/bot/gateways",
    tag = "Bots",
    responses(
        (status = 200, description = "Gateways", body = ListResponse<GatewayBotResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_gateways(
    State(_state): State<Arc<AppState>>,
    _service: VerifiedService,
) -> ApiResult<Json<ListResponse<GatewayBotResponse>>> {
    let items = vec![
        GatewayBotResponse {
            name: PaymentSystem::PlatformCard,
            display_name: "Платформа (Карта)".to_string(),
        },
        // GatewayResponse {
        //     name: PaymentSystem::PlatformSBP,
        //     display_name: "Платформа (СБП)".to_string(),
        // },
    ];

    // #[cfg(feature = "mock-payments-provider")]
    // items.push(GatewayResponse {
    //     name: PaymentSystem::Mock,
    //     display_name: "Криптоплатежи (мок-провайдер)".to_string(),
    // });

    Ok(Json(ListResponse {
        total: items.len() as i64,
        items,
    }))
}
