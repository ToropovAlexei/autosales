use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use axum_extra::extract::Multipart;
use bytes::Bytes;
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
        image::{CreateImage, ImageServiceTrait},
        payment_invoice::{
            CreatePaymentInvoiceCommand, PaymentInvoiceServiceTrait, SendInvoiceReceiptCommand,
            UpdatePaymentInvoiceCommand,
        },
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_invoice).get(list_invoices))
        .route("/{id}", get(get_invoice).patch(update_invoice))
        .route("/{id}/confirm", post(confirm_invoice))
        .route("/{id}/cancel", post(cancel_invoice))
        .route("/{id}/send-receipt", post(send_invoice_receipt))
}

#[utoipa::path(
    get,
    path = "/api/bot/invoices",
    tag = "Invoices",
    responses(
        (status = 200, description = "List of invoices", body = ListResponse<PaymentInvoiceResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/bot/invoices/{id}",
    tag = "Invoices",
    responses(
        (status = 200, description = "Invoice details", body = PaymentInvoiceResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<PaymentInvoiceResponse>> {
    let payment_invoice = state.payment_invoice_service.get_by_id(id).await?;
    Ok(Json(PaymentInvoiceResponse::from(payment_invoice)))
}

#[utoipa::path(
    post,
    path = "/api/bot/invoices",
    tag = "Invoices",
    request_body = NewPaymentInvoiceRequest,
    responses(
        (status = 200, description = "Invoice created", body = PaymentInvoiceResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
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

#[utoipa::path(
    patch,
    path = "/api/bot/invoices/{id}",
    tag = "Invoices",
    request_body = UpdatePaymentInvoiceRequest,
    responses(
        (status = 200, description = "Invoice updated", body = PaymentInvoiceResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn update_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<UpdatePaymentInvoiceRequest>,
) -> ApiResult<Json<PaymentInvoiceResponse>> {
    let invoice = state
        .payment_invoice_service
        .update(UpdatePaymentInvoiceCommand {
            id,
            notification_sent_at: payload.notification_sent_at,
            status: payload.status,
        })
        .await?;
    Ok(Json(PaymentInvoiceResponse::from(invoice)))
}

#[utoipa::path(
    post,
    path = "/api/bot/invoices/{id}/confirm",
    tag = "Invoices",
    responses(
        (status = 200, description = "Invoice confirmed", body = PaymentInvoiceResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn confirm_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<PaymentInvoiceResponse>> {
    let invoice = state.payment_invoice_service.confirm_invoice(id).await?;
    Ok(Json(PaymentInvoiceResponse::from(invoice)))
}

#[utoipa::path(
    post,
    path = "/api/bot/invoices/{id}/cancel",
    tag = "Invoices",
    responses(
        (status = 200, description = "Invoice cancelled", body = PaymentInvoiceResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn cancel_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<PaymentInvoiceResponse>> {
    let invoice = state.payment_invoice_service.cancel_invoice(id).await?;
    Ok(Json(PaymentInvoiceResponse::from(invoice)))
}

#[utoipa::path(
    post,
    path = "/api/bot/invoices/{id}/send-receipt",
    tag = "Invoices",
    responses(
        (status = 200, description = "Receipt submitted", body = PaymentInvoiceResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn send_invoice_receipt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
    mut multipart: Multipart,
) -> ApiResult<Json<PaymentInvoiceResponse>> {
    let file = get_receipt_from_form(&mut multipart)
        .await
        .map_err(ApiError::BadRequest)?;
    let _image = state
        .image_service
        .create(CreateImage {
            context: "invoice_receipt".to_string(),
            file,
            filename: "receipt".to_string(),
            created_by: 1, // System
        })
        .await?;

    let invoice = state
        .payment_invoice_service
        .send_invoice_receipt(SendInvoiceReceiptCommand {
            id,
            receipt_url: "https://dropmefiles.com/PwjmT".to_string(), // TODO TO BE IMPLEMENTED
        })
        .await?;
    Ok(Json(PaymentInvoiceResponse::from(invoice)))
}

async fn get_receipt_from_form(multipart: &mut Multipart) -> Result<Bytes, String> {
    let mut file: Option<Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| e.to_string())? {
        let name = field.name().ok_or("Field name missing")?.to_string();
        if name.as_str() == "file" {
            let data = field.bytes().await.map_err(|e| e.to_string())?;
            file = Some(data);
        }
    }

    let file = file.ok_or("Missing 'file' field")?;

    Ok(file)
}
