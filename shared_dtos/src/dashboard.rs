use chrono::NaiveDate;
use serde::Serialize;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "dashboard.ts", rename = "StatWithTrend")
)]
#[derive(Debug, Clone, Serialize)]
pub struct StatWithTrendResponse {
    pub value: i64,
    pub trend: f64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "dashboard.ts", rename = "DashboardOverview")
)]
#[derive(Debug, Clone, Serialize)]
pub struct DashboardOverviewResponse {
    pub total_users: i64,
    pub users_with_purchases: i64,
    pub available_products: i64,
    pub total_users_30_days: StatWithTrendResponse,
    pub users_with_purchases_30_days: StatWithTrendResponse,
    pub products_sold_30_days: StatWithTrendResponse,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "dashboard.ts", rename = "TimeSeriesPoint")
)]
#[derive(Debug, Clone, Serialize)]
pub struct TimeSeriesPointResponse {
    pub date: NaiveDate,
    pub value: i64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "dashboard.ts", rename = "SalesOverTime")
)]
#[derive(Debug, Clone, Serialize)]
pub struct SalesOverTimeResponse {
    pub products_sold: i64,
    pub total_revenue: f64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "dashboard.ts", rename = "TimeSeriesDashboardData")
)]
#[derive(Debug, Clone, Serialize)]
pub struct TimeSeriesDashboardDataResponse {
    pub sales: SalesOverTimeResponse,
    pub sales_chart: Vec<TimeSeriesPointResponse>,
    pub users_chart: Vec<TimeSeriesPointResponse>,
    pub revenue_chart: Vec<TimeSeriesPointResponse>,
    pub users_with_purchases_chart: Vec<TimeSeriesPointResponse>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "dashboard.ts", rename = "TopProduct")
)]
#[derive(Debug, Clone, Serialize)]
pub struct TopProductResponse {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub total_revenue: f64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "dashboard.ts", rename = "CategorySales")
)]
#[derive(Debug, Clone, Serialize)]
pub struct CategorySalesResponse {
    pub category_name: String,
    pub total_sales: f64,
}
