use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

use crate::{
    errors::api::{ApiError, ApiResult},
    middlewares::{bot_auth::AuthBot, validator::ValidatedJson},
    models::payment_invoice::PaymentInvoiceListQuery,
    presentation::{
        admin::dtos::list_response::ListResponse,
        bot::dtos::invoice::{
            NewPaymentInvoiceRequest, PaymentInvoiceResponse, UpdatePaymentInvoiceRequest,
        },
    },
    services::{
        customer::CustomerServiceTrait,
        payment_invoice::{
            CreatePaymentInvoiceCommand, PaymentInvoiceServiceTrait, UpdatePaymentInvoiceCommand,
        },
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_invoice).get(list_invoices))
        .route("/{id}", get(get_invoice).patch(update_invoice))
}

async fn list_invoices(
    State(state): State<Arc<AppState>>,
    _bot: AuthBot,
    query: PaymentInvoiceListQuery,
) -> ApiResult<Json<ListResponse<PaymentInvoiceResponse>>> {
    let payment_invoices = state.payment_invoice_service.get_list(query).await?;
    Ok(Json(ListResponse {
        total: payment_invoices.total,
        items: payment_invoices
            .items
            .into_iter()
            .map(PaymentInvoiceResponse::from)
            .collect(),
    }))
}

async fn get_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<PaymentInvoiceResponse>> {
    let payment_invoice = state.payment_invoice_service.get_by_id(id).await?;
    Ok(Json(PaymentInvoiceResponse::from(payment_invoice)))
}

async fn create_invoice(
    State(state): State<Arc<AppState>>,
    _bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<NewPaymentInvoiceRequest>,
) -> ApiResult<Json<PaymentInvoiceResponse>> {
    let customer_id = state
        .customer_service
        .get_by_telegram_id(payload.telegram_id)
        .await
        .map(|c| c.id)?;
    let payment_invoice = state
        .payment_invoice_service
        .create(CreatePaymentInvoiceCommand {
            amount: Decimal::from_f64(payload.amount)
                .ok_or(ApiError::BadRequest("Failed to parse amount".to_string()))?,
            customer_id,
            gateway: payload.gateway,
        })
        .await?;
    Ok(Json(PaymentInvoiceResponse::from(payment_invoice)))
}

async fn update_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<UpdatePaymentInvoiceRequest>,
) -> ApiResult<Json<PaymentInvoiceResponse>> {
    let customer = state
        .payment_invoice_service
        .update(UpdatePaymentInvoiceCommand {
            id,
            notification_sent_at: payload.notification_sent_at,
            status: payload.status,
        })
        .await?;
    Ok(Json(PaymentInvoiceResponse::from(customer)))
}
