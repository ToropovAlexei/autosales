use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "stock_movement.ts"))]
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StockMovementType {
    Initial,
    Restock,
    Sale,
    Return,
    Adjustment,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "stock_movement.ts", rename = "StockMovement")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovementAdminResponse {
    pub id: i64,
    pub order_id: Option<i64>,
    pub product_id: i64,
    pub product_name: String,
    pub r#type: StockMovementType,
    pub quantity: i64,
    pub created_at: DateTime<Utc>,
    pub created_by: i64,
    pub description: Option<String>,
}
