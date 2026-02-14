use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use shared_dtos::{
    analytics::BotAnalyticsBotResponse,
    customer::{CustomerBotResponse, NewCustomerBotRequest, UpdateCustomerBotRequest},
    invoice::PaymentInvoiceBotResponse,
    list_response::ListResponse,
    order::EnrichedOrderBotResponse,
    user_subscription::UserSubscriptionBotResponse,
};

use crate::{
    errors::api::{ApiResult, ErrorResponse},
    middlewares::{bot_auth::AuthBot, validator::ValidatedJson},
    models::customer::NewCustomer,
    services::{
        analytics::AnalyticsServiceTrait,
        customer::{CustomerServiceTrait, UpdateCustomerCommand},
        order::OrderServiceTrait,
        payment_invoice::PaymentInvoiceServiceTrait,
        user_subscription::UserSubscriptionServiceTrait,
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
            "/{telegram_id}/subscriptions",
            get(get_customer_subscriptions),
        )
        .route(
            "/{telegram_id}/update-last-seen",
            post(update_customer_last_seen),
        )
        .route(
            "/{telegram_id}/referral-analytics",
            get(get_customer_referral_analytics),
        )
}

#[utoipa::path(
    get,
    path = "/api/bot/customers/{telegram_id}",
    tag = "Customers",
    responses(
        (status = 200, description = "Get customer", body = CustomerBotResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
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
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn create_customer(
    State(state): State<Arc<AppState>>,
    bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<NewCustomerBotRequest>,
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
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn update_customer(
    State(state): State<Arc<AppState>>,
    Path(telegram_id): Path<i64>,
    _bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<UpdateCustomerBotRequest>,
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
            blocked_until: None,
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
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
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
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
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
    post,
    path = "/api/bot/customers/{telegram_id}/update-last-seen",
    tag = "Customers",
    responses(
        (status = 200, description = "Update last seen"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
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

#[utoipa::path(
    get,
    path = "/api/bot/customers/{telegram_id}/subscriptions",
    tag = "Customers",
    responses(
        (status = 200, description = "Get customer subscriptions", body = ListResponse<UserSubscriptionBotResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn get_customer_subscriptions(
    State(state): State<Arc<AppState>>,
    Path(telegram_id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<ListResponse<UserSubscriptionBotResponse>>> {
    let customer_id = state
        .customer_service
        .get_by_telegram_id(telegram_id)
        .await?;
    let subscriptions = state
        .user_subscription_service
        .get_for_customer(customer_id.id)
        .await?;

    Ok(Json(ListResponse {
        total: subscriptions.len() as i64,
        items: subscriptions
            .into_iter()
            .map(UserSubscriptionBotResponse::from)
            .collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/bot/customers/{telegram_id}/referral-analytics",
    tag = "Customers",
    responses(
        (status = 200, description = "Get customer referral analytics", body = ListResponse<BotAnalyticsBotResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
async fn get_customer_referral_analytics(
    State(state): State<Arc<AppState>>,
    Path(telegram_id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<ListResponse<BotAnalyticsBotResponse>>> {
    let customer = state
        .customer_service
        .get_by_telegram_id(telegram_id)
        .await?;
    let stats = state
        .analytics_service
        .get_referral_stats(customer.id)
        .await?;
    Ok(Json(ListResponse {
        total: stats.len() as i64,
        items: stats
            .into_iter()
            .map(BotAnalyticsBotResponse::from)
            .collect(),
    }))
}
