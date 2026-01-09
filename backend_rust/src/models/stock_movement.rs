use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::define_list_query;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Deserialize, Serialize, TS, ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "stock_movement.ts")]
pub enum StockMovementType {
    Initial,
    Restock,
    Sale,
    Return,
    Adjustment,
}

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct StockMovementRow {
    pub id: i64,
    pub order_id: Option<i64>,
    pub product_id: i64,
    pub product_name: String,
    pub r#type: StockMovementType,
    pub quantity: i64,
    pub balance_after: i64,
    pub created_at: DateTime<Utc>,
    pub created_by: i64,
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
    pub description: Option<String>,
    pub reference_id: Option<String>,
}

define_list_query! {
    query_name: StockMovementListQuery,
    filter_fields: {
        StockMovementFilterFields,
        [
            Id => "id",
            OrderId => "order_id",
            ProductId => "product_id",
            Type => "type",
            Quantity => "quantity",
            CreatedAt => "created_at",
            CreatedBy => "created_by"
        ]
    },
    order_fields: {
        StockMovementOrderFields,
        [
            Id => "id",
            OrderId => "order_id",
            ProductId => "product_id",
            Type => "type",
            Quantity => "quantity",
            CreatedAt => "created_at",
            CreatedBy => "created_by"
        ]
    }
}
