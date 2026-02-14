use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_dtos::stock_movement::StockMovementType;
use sqlx::prelude::FromRow;

use crate::define_list_query;

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
