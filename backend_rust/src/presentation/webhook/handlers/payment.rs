use axum::Json;
use std::sync::Arc;
use uuid::Uuid;

use axum::{Router, extract::State, routing::post};

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::external::payment::mock::{
        MockPaymentsProviderTrait, dto::MockProviderInvoiceWebhookPayload,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    let router = Router::new();
    #[cfg(feature = "mock-payments-provider")]
    let router = router.route("/mock-provider", post(mock_payments_provider_webhook));

    router
}

#[cfg(feature = "mock-payments-provider")]
async fn mock_payments_provider_webhook(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MockProviderInvoiceWebhookPayload>,
) -> ApiResult<Json<Uuid>> {
    let order_id = state
        .mock_payments_provider
        .handle_webhook(payload)
        .await
        .map_err(ApiError::BadRequest)?;
    Ok(Json(order_id))
}
