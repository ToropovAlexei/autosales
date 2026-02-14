use shared_dtos::{error::ApiErrorResponse, list_response::ListResponse};
use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::{
    errors::api::ApiResult,
    middlewares::require_permission::{InvoicesRead, RequirePermission},
    models::payment_invoice::PaymentInvoiceListQuery,
    presentation::admin::dtos::payment_invoice::PaymentInvoiceResponse,
    services::{auth::AuthUser, payment_invoice::PaymentInvoiceServiceTrait},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(list_payment_invoices))
}

#[utoipa::path(
    get,
    path = "/api/admin/payment-invoices",
    tag = "Payment Invoices",
    responses(
        (status = 200, description = "Payment invoice list", body = ListResponse<PaymentInvoiceResponse>),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
async fn list_payment_invoices(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<InvoicesRead>,
    query: PaymentInvoiceListQuery,
) -> ApiResult<Json<ListResponse<PaymentInvoiceResponse>>> {
    let invoices = state.payment_invoice_service.get_list(query).await?;

    Ok(Json(ListResponse {
        total: invoices.total,
        items: invoices
            .items
            .into_iter()
            .map(PaymentInvoiceResponse::from)
            .collect(),
    }))
}
