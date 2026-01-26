use axum::Json;
use std::sync::Arc;
use uuid::Uuid;

use axum::{Router, extract::State, routing::post};

#[cfg(feature = "mock-payments-provider")]
use crate::infrastructure::external::payment::mock::{
    MockPaymentsProviderTrait, dto::MockProviderInvoiceWebhookPayload,
};

use crate::{
    errors::api::{ApiError, ApiResult},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    let router = Router::new();
    #[cfg(feature = "mock-payments-provider")]
    let router = router.route("/mock-provider", post(mock_payments_provider_webhook));

    router
}

#[cfg(feature = "mock-payments-provider")]
#[utoipa::path(
    post,
    path = "/api/webhook/payment/mock-provider",
    tag = "Webhook",
    request_body = MockProviderInvoiceWebhookPayload,
    responses(
        (status = 200, description = "Webhook accepted", body = Uuid),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn mock_payments_provider_webhook(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MockProviderInvoiceWebhookPayload>,
) -> ApiResult<Json<Uuid>> {
    use crate::services::payment_processing_service::PaymentProcessingServiceTrait;

    let order_id = state
        .mock_payments_provider
        .handle_webhook(payload)
        .await
        .map_err(ApiError::BadRequest)?;
    state
        .payment_processing_service
        .handle_payment_success(order_id)
        .await?;
    Ok(Json(order_id))
}
