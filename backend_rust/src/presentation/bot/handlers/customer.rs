use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use shared_dtos::{
    customer::CustomerBotResponse, invoice::PaymentInvoiceBotResponse,
    order::EnrichedOrderBotResponse,
};

use crate::{
    errors::api::ApiResult,
    middlewares::{bot_auth::AuthBot, validator::ValidatedJson},
    models::customer::NewCustomer,
    presentation::{
        admin::dtos::list_response::ListResponse,
        bot::dtos::customer::{NewCustomerRequest, UpdateCustomerRequest},
    },
    services::{
        customer::{CustomerServiceTrait, UpdateCustomerCommand},
        order::OrderServiceTrait,
        payment_invoice::PaymentInvoiceServiceTrait,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_customer))
        .route("/{telegram_id}", get(get_customer).patch(update_customer))
        .route("/{telegram_id}/invoices", get(get_customer_invoices))
        .route("/{telegram_id}/orders", get(get_customer_orders))
        .route(
            "/{telegram_id}/update-last-seen",
            post(update_customer_last_seen),
        )
}

#[utoipa::path(
    get,
    path = "/api/bot/customers/{telegram_id}",
    tag = "Customers",
    responses(
        (status = 200, description = "Get customer", body = CustomerBotResponse),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_customer(
    State(state): State<Arc<AppState>>,
    Path(telegram_id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<CustomerBotResponse>> {
    let customer = state
        .customer_service
        .get_by_telegram_id(telegram_id)
        .await?;
    Ok(Json(CustomerBotResponse::from(customer)))
}

#[utoipa::path(
    post,
    path = "/api/bot/customers",
    tag = "Customers",
    responses(
        (status = 200, description = "Create customer", body = CustomerBotResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn create_customer(
    State(state): State<Arc<AppState>>,
    bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<NewCustomerRequest>,
) -> ApiResult<Json<CustomerBotResponse>> {
    let customer = state
        .customer_service
        .create(NewCustomer {
            telegram_id: payload.telegram_id,
            registered_with_bot: bot.bot_id,
        })
        .await?;
    Ok(Json(CustomerBotResponse::from(customer)))
}

#[utoipa::path(
    patch,
    path = "/api/bot/customers/{telegram_id}",
    tag = "Customers",
    responses(
        (status = 200, description = "Update customer", body = CustomerBotResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn update_customer(
    State(state): State<Arc<AppState>>,
    Path(telegram_id): Path<i64>,
    _bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<UpdateCustomerRequest>,
) -> ApiResult<Json<CustomerBotResponse>> {
    let prev = state
        .customer_service
        .get_by_telegram_id(telegram_id)
        .await?;
    let customer = state
        .customer_service
        .update(UpdateCustomerCommand {
            bot_is_blocked_by_user: payload.bot_is_blocked_by_user,
            has_passed_captcha: payload.has_passed_captcha,
            id: prev.id,
            is_blocked: None,
            last_seen_at: None,
            updated_by: None,
            last_seen_with_bot: None,
            ctx: None,
        })
        .await?;
    Ok(Json(CustomerBotResponse::from(customer)))
}

#[utoipa::path(
    get,
    path = "/api/bot/customers/{telegram_id}/invoices",
    tag = "Customers",
    responses(
        (status = 200, description = "Get customer invoices", body = ListResponse<PaymentInvoiceBotResponse>),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_customer_invoices(
    State(state): State<Arc<AppState>>,
    Path(telegram_id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<ListResponse<PaymentInvoiceBotResponse>>> {
    let customer_id = state
        .customer_service
        .get_by_telegram_id(telegram_id)
        .await?;
    let invoices = state
        .payment_invoice_service
        .get_for_customer(customer_id.id)
        .await?;

    Ok(Json(ListResponse {
        total: invoices.len() as i64,
        items: invoices
            .into_iter()
            .map(PaymentInvoiceBotResponse::from)
            .collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/bot/customers/{telegram_id}/orders",
    tag = "Customers",
    responses(
        (status = 200, description = "Get customer orders", body = ListResponse<EnrichedOrderBotResponse>),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_customer_orders(
    State(state): State<Arc<AppState>>,
    Path(telegram_id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<ListResponse<EnrichedOrderBotResponse>>> {
    let customer_id = state
        .customer_service
        .get_by_telegram_id(telegram_id)
        .await?;
    let orders = state.order_service.get_for_customer(customer_id.id).await?;

    Ok(Json(ListResponse {
        total: orders.len() as i64,
        items: orders
            .into_iter()
            .map(EnrichedOrderBotResponse::from)
            .collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/bot/customers/{telegram_id}/update-last-seen",
    tag = "Customers",
    responses(
        (status = 200, description = "Update last seen"),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn update_customer_last_seen(
    State(state): State<Arc<AppState>>,
    Path(telegram_id): Path<i64>,
    bot: AuthBot,
) -> ApiResult<Json<()>> {
    let customer = state
        .customer_service
        .get_by_telegram_id(telegram_id)
        .await?;
    state
        .customer_service
        .update_last_seen(customer.id, bot.bot_id)
        .await?;

    Ok(Json(()))
}
