use chrono::NaiveDate;
use serde::Serialize;
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};

#[derive(Debug, Clone, Serialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "dashboard.ts", rename = "StatWithTrend")]
pub struct StatWithTrendResponse {
    pub value: i64,
    pub trend: f64,
}

#[derive(Debug, Clone, Serialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "dashboard.ts", rename = "DashboardOverview")]
pub struct DashboardOverviewResponse {
    pub total_users: i64,
    pub users_with_purchases: i64,
    pub available_products: i64,
    pub total_users_30_days: StatWithTrendResponse,
    pub users_with_purchases_30_days: StatWithTrendResponse,
    pub products_sold_30_days: StatWithTrendResponse,
}

#[derive(Debug, Clone, Serialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "dashboard.ts", rename = "TimeSeriesPoint")]
pub struct TimeSeriesPointResponse {
    pub date: NaiveDate,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "dashboard.ts", rename = "SalesOverTime")]
pub struct SalesOverTimeResponse {
    pub products_sold: i64,
    pub total_revenue: f64,
}

#[derive(Debug, Clone, Serialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "dashboard.ts", rename = "TimeSeriesDashboardData")]
pub struct TimeSeriesDashboardDataResponse {
    pub sales: SalesOverTimeResponse,
    pub sales_chart: Vec<TimeSeriesPointResponse>,
    pub users_chart: Vec<TimeSeriesPointResponse>,
    pub revenue_chart: Vec<TimeSeriesPointResponse>,
    pub users_with_purchases_chart: Vec<TimeSeriesPointResponse>,
}

#[derive(Debug, Clone, Serialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "dashboard.ts", rename = "TopProduct")]
pub struct TopProductResponse {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub total_revenue: f64,
}

#[derive(Debug, Clone, Serialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "dashboard.ts", rename = "CategorySales")]
pub struct CategorySalesResponse {
    pub category_name: String,
    pub total_sales: f64,
}
