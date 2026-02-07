use shared_dtos::list_response::ListResponse;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::{
    errors::api::ApiResult,
    middlewares::{
        context::RequestContext,
        require_permission::{CustomersRead, CustomersUpdate, RequirePermission},
        validator::ValidatedJson,
    },
    models::customer::CustomerListQuery,
    presentation::admin::dtos::customer::{CustomerResponse, UpdateCustomerRequest},
    services::{
        auth::AuthUser,
        customer::{CustomerServiceTrait, UpdateCustomerCommand},
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_customers))
        .route("/{id}", get(get_customer).patch(update_customer))
}

#[utoipa::path(
    get,
    path = "/api/admin/customers",
    tag = "Customers",
    responses(
        (status = 200, description = "Customers list", body = ListResponse<CustomerResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn list_customers(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<CustomersRead>,
    query: CustomerListQuery,
) -> ApiResult<Json<ListResponse<CustomerResponse>>> {
    let customers = state.customer_service.get_list(query).await?;

    Ok(Json(ListResponse {
        total: customers.total,
        items: customers
            .items
            .into_iter()
            .map(CustomerResponse::from)
            .collect(),
    }))
}

#[utoipa::path(
    patch,
    path = "/api/admin/customers/{id}",
    tag = "Customers",
    responses(
        (status = 200, description = "Customer updated", body = CustomerResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 404, description = "Customer not found", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn update_customer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    user: AuthUser,
    _perm: RequirePermission<CustomersUpdate>,
    ctx: RequestContext,
    ValidatedJson(payload): ValidatedJson<UpdateCustomerRequest>,
) -> ApiResult<Json<CustomerResponse>> {
    let customer = state
        .customer_service
        .update(UpdateCustomerCommand {
            id,
            updated_by: Some(user.id),
            is_blocked: payload.is_blocked,
            bot_is_blocked_by_user: None,
            has_passed_captcha: None,
            last_seen_at: None,
            last_seen_with_bot: None,
            ctx: Some(ctx),
            blocked_until: None,
        })
        .await?;

    Ok(Json(CustomerResponse::from(customer)))
}

#[utoipa::path(
    get,
    path = "/api/admin/customers/{id}",
    tag = "Customers",
    responses(
        (status = 200, description = "Customer", body = CustomerResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 404, description = "Customer not found", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_customer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<CustomersRead>,
) -> ApiResult<Json<CustomerResponse>> {
    let customer = state.customer_service.get_by_id(id).await?;

    Ok(Json(CustomerResponse::from(customer)))
}
