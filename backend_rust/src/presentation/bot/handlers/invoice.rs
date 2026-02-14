use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use axum_extra::extract::Multipart;
use bytes::Bytes;
use reqwest::multipart;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};
use shared_dtos::{
    error::ApiErrorResponse,
    invoice::{
        NewPaymentInvoiceBotRequest, PaymentInvoiceBotResponse, UpdatePaymentInvoiceBotRequest,
    },
    list_response::ListResponse,
};
use utoipa::ToSchema;

use crate::{
    errors::api::{ApiError, ApiResult},
    middlewares::{bot_auth::AuthBot, validator::ValidatedJson},
    models::payment_invoice::PaymentInvoiceListQuery,
    services::{
        customer::CustomerServiceTrait,
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

#[derive(ToSchema)]
#[allow(dead_code)] // Used by utoipa request_body schema only.
struct SendInvoiceReceiptMultipartRequest {
    #[schema(value_type = String, format = Binary)]
    file: String,
}

#[utoipa::path(
    get,
    path = "/api/bot/invoices",
    tag = "Invoices",
    responses(
        (status = 200, description = "List of invoices", body = ListResponse<PaymentInvoiceBotResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn list_invoices(
    State(state): State<Arc<AppState>>,
    _bot: AuthBot,
    query: PaymentInvoiceListQuery,
) -> ApiResult<Json<ListResponse<PaymentInvoiceBotResponse>>> {
    let payment_invoices = state.payment_invoice_service.get_list(query).await?;
    Ok(Json(ListResponse {
        total: payment_invoices.total,
        items: payment_invoices
            .items
            .into_iter()
            .map(PaymentInvoiceBotResponse::from)
            .collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/bot/invoices/{id}",
    tag = "Invoices",
    responses(
        (status = 200, description = "Invoice details", body = PaymentInvoiceBotResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn get_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<PaymentInvoiceBotResponse>> {
    let payment_invoice = state.payment_invoice_service.get_by_id(id).await?;
    Ok(Json(PaymentInvoiceBotResponse::from(payment_invoice)))
}

#[utoipa::path(
    post,
    path = "/api/bot/invoices",
    tag = "Invoices",
    request_body = NewPaymentInvoiceBotRequest,
    responses(
        (status = 200, description = "Invoice created", body = PaymentInvoiceBotResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn create_invoice(
    State(state): State<Arc<AppState>>,
    _bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<NewPaymentInvoiceBotRequest>,
) -> ApiResult<Json<PaymentInvoiceBotResponse>> {
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
    Ok(Json(PaymentInvoiceBotResponse::from(payment_invoice)))
}

#[utoipa::path(
    patch,
    path = "/api/bot/invoices/{id}",
    tag = "Invoices",
    request_body = UpdatePaymentInvoiceBotRequest,
    responses(
        (status = 200, description = "Invoice updated", body = PaymentInvoiceBotResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn update_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<UpdatePaymentInvoiceBotRequest>,
) -> ApiResult<Json<PaymentInvoiceBotResponse>> {
    let invoice = state
        .payment_invoice_service
        .update(UpdatePaymentInvoiceCommand {
            id,
            notification_sent_at: payload.notification_sent_at,
            status: payload.status,
            ..Default::default()
        })
        .await?;
    Ok(Json(PaymentInvoiceBotResponse::from(invoice)))
}

#[utoipa::path(
    post,
    path = "/api/bot/invoices/{id}/confirm",
    tag = "Invoices",
    responses(
        (status = 200, description = "Invoice confirmed", body = PaymentInvoiceBotResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn confirm_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<PaymentInvoiceBotResponse>> {
    let invoice = state.payment_invoice_service.confirm_invoice(id).await?;
    Ok(Json(PaymentInvoiceBotResponse::from(invoice)))
}

#[utoipa::path(
    post,
    path = "/api/bot/invoices/{id}/cancel",
    tag = "Invoices",
    responses(
        (status = 200, description = "Invoice cancelled", body = PaymentInvoiceBotResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn cancel_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<PaymentInvoiceBotResponse>> {
    let invoice = state.payment_invoice_service.cancel_invoice(id).await?;
    Ok(Json(PaymentInvoiceBotResponse::from(invoice)))
}

#[utoipa::path(
    post,
    path = "/api/bot/invoices/{id}/send-receipt",
    tag = "Invoices",
    request_body(content = SendInvoiceReceiptMultipartRequest, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Receipt submitted", body = PaymentInvoiceBotResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn send_invoice_receipt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _bot: AuthBot,
    mut multipart: Multipart,
) -> ApiResult<Json<PaymentInvoiceBotResponse>> {
    let file = get_receipt_from_form(&mut multipart)
        .await
        .map_err(ApiError::BadRequest)?;

    let uploaded_file = upload_file_to_files_fm(
        file,
        state.client.clone(),
        &state.config.files_fm_folder_hash,
        &state.config.files_fm_upload_token,
    )
    .await
    .map_err(ApiError::InternalServerError)?;

    let invoice = state
        .payment_invoice_service
        .send_invoice_receipt(SendInvoiceReceiptCommand {
            id,
            receipt_url: format!("https://files.fm/f/{uploaded_file}"),
        })
        .await?;
    Ok(Json(PaymentInvoiceBotResponse::from(invoice)))
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FilesFmResponse {
    pub result: String,
    pub file_hash: String,
}

async fn upload_file_to_files_fm(
    bytes: Bytes,
    client: Arc<reqwest::Client>,
    folder_hash: &str,
    upload_token: &str,
) -> Result<String, String> {
    let part = multipart::Part::bytes(bytes.to_vec())
        .file_name("receipt.pdf")
        .mime_str("application/pdf")
        .map_err(|e| e.to_string())?;
    let form = multipart::Form::new().part("file", part);
    let response = client
        .post(format!(
            "https://api.files.fm/save_file.php?up_id={folder_hash}&key={upload_token}&get_file_hash"
        ))
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let response = response
        .json::<FilesFmResponse>()
        .await
        .map_err(|e| e.to_string())?;
    if response.file_hash.is_empty() {
        return Err(format!("Empty hash. Result: {}", response.result));
    }
    Ok(response.file_hash)
}
