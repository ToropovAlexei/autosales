use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};

use crate::models::stock_movement::{StockMovementRow, StockMovementType};

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "stock_movement.ts", rename = "StockMovement")]
pub struct StockMovementResponse {
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

impl From<StockMovementRow> for StockMovementResponse {
    fn from(r: StockMovementRow) -> Self {
        StockMovementResponse {
            id: r.id,
            order_id: r.order_id,
            product_id: r.product_id,
            product_name: r.product_name,
            r#type: r.r#type,
            quantity: r.quantity,
            created_at: r.created_at,
            created_by: r.created_by,
            description: r.description,
        }
    }
}
