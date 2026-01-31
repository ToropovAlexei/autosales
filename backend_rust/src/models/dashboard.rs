use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct TimeSeriesRow {
    pub date: NaiveDate,
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct TopProductRow {
    pub id: i64,
    pub name: String,
    pub price: Decimal,
    pub total_revenue: Decimal,
}

#[derive(Debug, Clone)]
pub struct CategorySalesRow {
    pub category_name: Option<String>,
    pub total_sales: Decimal,
}
