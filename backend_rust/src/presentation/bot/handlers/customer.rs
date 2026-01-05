use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

use crate::{
    errors::api::ApiResult,
    middlewares::{bot_auth::AuthBot, validator::ValidatedJson},
    models::customer::NewCustomer,
    presentation::bot::dtos::customer::{
        CustomerResponse, NewCustomerRequest, UpdateCustomerRequest,
    },
    services::customer::{CustomerServiceTrait, UpdateCustomerCommand},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_customer))
        .route("/{telegram_id}", get(get_customer).patch(update_customer))
}

#[utoipa::path(
    get,
    path = "/api/bot/customers/{telegram_id}",
    tag = "Customers",
    responses(
        (status = 200, description = "Get customer", body = CustomerResponse),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_customer(
    State(state): State<Arc<AppState>>,
    Path(telegram_id): Path<i64>,
    _bot: AuthBot,
) -> ApiResult<Json<CustomerResponse>> {
    let customer = state
        .customer_service
        .get_by_telegram_id(telegram_id)
        .await?;
    Ok(Json(CustomerResponse::from(customer)))
}

#[utoipa::path(
    post,
    path = "/api/bot/customers",
    tag = "Customers",
    responses(
        (status = 200, description = "Create customer", body = CustomerResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn create_customer(
    State(state): State<Arc<AppState>>,
    bot: AuthBot,
    ValidatedJson(payload): ValidatedJson<NewCustomerRequest>,
) -> ApiResult<Json<CustomerResponse>> {
    let customer = state
        .customer_service
        .create(NewCustomer {
            telegram_id: payload.telegram_id,
            registered_with_bot: bot.bot_id,
        })
        .await?;
    Ok(Json(CustomerResponse::from(customer)))
}

#[utoipa::path(
    patch,
    path = "/api/bot/customers/{telegram_id}",
    tag = "Customers",
    responses(
        (status = 200, description = "Update customer", body = CustomerResponse),
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
) -> ApiResult<Json<CustomerResponse>> {
    let customer = state
        .customer_service
        .update(UpdateCustomerCommand {
            bot_is_blocked_by_user: payload.bot_is_blocked_by_user,
            has_passed_captcha: payload.has_passed_captcha,
            id: telegram_id,
            is_blocked: None,
            last_seen_at: None,
            updated_by: None,
            last_seen_with_bot: None,
            ctx: None,
        })
        .await?;
    Ok(Json(CustomerResponse::from(customer)))
}
