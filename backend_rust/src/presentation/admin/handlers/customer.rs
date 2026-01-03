use axum::routing::patch;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::{
    errors::api::ApiResult,
    middlewares::{
        require_permission::{CustomersRead, CustomersUpdate, RequirePermission},
        validator::ValidatedJson,
    },
    models::customer::{CustomerListQuery, UpdateCustomer},
    presentation::admin::dtos::{
        customer::{CustomerResponse, UpdateCustomerRequest},
        list_response::ListResponse,
    },
    services::{auth::AuthUser, customer::CustomerServiceTrait},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_customers))
        .route("/{id}", patch(update_customer))
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
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn update_customer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    _user: AuthUser,
    _perm: RequirePermission<CustomersUpdate>,
    ValidatedJson(payload): ValidatedJson<UpdateCustomerRequest>,
) -> ApiResult<Json<CustomerResponse>> {
    let category = state
        .customer_service
        .update(
            id,
            UpdateCustomer {
                is_blocked: payload.is_blocked,
                bot_is_blocked_by_user: None,
                has_passed_captcha: None,
                last_seen_at: None,
                last_seen_with_bot: None,
            },
        )
        .await?;

    Ok(Json(category.into()))
}
