use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    errors::api::{ApiError, ApiResult},
    middlewares::require_permission::{DashboardRead, RequirePermission},
    presentation::admin::dtos::dashboard::{
        CategorySalesResponse, DashboardOverviewResponse, TimeSeriesDashboardDataResponse,
        TopProductResponse,
    },
    services::{auth::AuthUser, dashboard::DashboardServiceTrait},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/stats", get(get_dashboard_stats))
        .route("/time-series", get(get_time_series))
        .route("/top-products", get(get_top_products))
        .route("/sales-by-category", get(get_sales_by_category))
}

#[derive(Debug, Deserialize)]
struct TimeSeriesQuery {
    start_date: String,
    end_date: String,
}

#[utoipa::path(
    get,
    path = "/api/admin/dashboard/stats",
    tag = "Dashboard",
    responses(
        (status = 200, description = "Dashboard stats", body = DashboardOverviewResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_dashboard_stats(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<DashboardRead>,
) -> ApiResult<Json<DashboardOverviewResponse>> {
    let stats = state.dashboard_service.get_dashboard_stats().await?;
    Ok(Json(stats))
}

#[utoipa::path(
    get,
    path = "/api/admin/dashboard/time-series",
    tag = "Dashboard",
    params(
        ("start_date" = String, Query, description = "RFC3339 start date"),
        ("end_date" = String, Query, description = "RFC3339 end date")
    ),
    responses(
        (status = 200, description = "Time series data", body = TimeSeriesDashboardDataResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_time_series(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<DashboardRead>,
    Query(query): Query<TimeSeriesQuery>,
) -> ApiResult<Json<TimeSeriesDashboardDataResponse>> {
    let start = DateTime::parse_from_rfc3339(&query.start_date)
        .map_err(|_| ApiError::BadRequest("Invalid start_date format".to_string()))?
        .with_timezone(&Utc);
    let end = DateTime::parse_from_rfc3339(&query.end_date)
        .map_err(|_| ApiError::BadRequest("Invalid end_date format".to_string()))?
        .with_timezone(&Utc);

    let data = state
        .dashboard_service
        .get_time_series_data(start, end)
        .await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/admin/dashboard/top-products",
    tag = "Dashboard",
    responses(
        (status = 200, description = "Top products", body = Vec<TopProductResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_top_products(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<DashboardRead>,
) -> ApiResult<Json<Vec<TopProductResponse>>> {
    let products = state.dashboard_service.get_top_products(5).await?;
    Ok(Json(products))
}

#[utoipa::path(
    get,
    path = "/api/admin/dashboard/sales-by-category",
    tag = "Dashboard",
    responses(
        (status = 200, description = "Sales by category", body = Vec<CategorySalesResponse>),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_sales_by_category(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    _perm: RequirePermission<DashboardRead>,
) -> ApiResult<Json<Vec<CategorySalesResponse>>> {
    let categories = state.dashboard_service.get_sales_by_category().await?;
    Ok(Json(categories))
}
