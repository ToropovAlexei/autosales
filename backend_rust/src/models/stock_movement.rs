use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Deserialize, Serialize, TS, ToSchema)]
#[sqlx(type_name = "stock_movement_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "stock_movement.ts")]
pub enum StockMovementType {
    Initial,
    Restock,
    Sale,
    Return,
    Adjustment,
}

#[derive(FromRow, Debug)]
pub struct StockMovementRow {
    pub id: i64,
    pub order_id: Option<i64>,
    pub product_id: i64,
    pub r#type: StockMovementType,
    pub quantity: i64,
    pub balance_after: i64,
    pub created_at: DateTime<Utc>,
    pub created_by: i64,
    pub source: String,
    pub description: Option<String>,
    pub reference_id: Option<String>,
}

#[derive(Debug)]
pub struct NewStockMovement {
    pub order_id: Option<i64>,
    pub product_id: i64,
    pub r#type: StockMovementType,
    pub quantity: i64,
    pub created_by: i64,
    pub source: String,
    pub description: Option<String>,
    pub reference_id: Option<String>,
}
