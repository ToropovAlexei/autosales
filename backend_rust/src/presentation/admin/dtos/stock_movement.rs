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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_movement_response_from_stock_movement_row_full() {
        let now = Utc::now();
        let row = StockMovementRow {
            id: 1,
            order_id: Some(10),
            product_id: 100,
            product_name: "Test Product".to_string(),
            r#type: StockMovementType::Adjustment,
            quantity: 5,
            created_at: now,
            created_by: 1,
            description: Some("Initial stock".to_string()),
            balance_after: 5,
            reference_id: None,
        };

        let response: StockMovementResponse = row.into();

        assert_eq!(response.id, 1);
        assert_eq!(response.order_id, Some(10));
        assert_eq!(response.product_id, 100);
        assert_eq!(response.product_name, "Test Product");
        assert_eq!(response.r#type, StockMovementType::Adjustment);
        assert_eq!(response.quantity, 5);
        assert_eq!(response.created_at, now);
        assert_eq!(response.created_by, 1);
        assert_eq!(response.description, Some("Initial stock".to_string()));
    }

    #[test]
    fn test_stock_movement_response_from_stock_movement_row_minimal() {
        let now = Utc::now();
        let row = StockMovementRow {
            id: 2,
            order_id: None,
            product_id: 101,
            product_name: "Another Product".to_string(),
            r#type: StockMovementType::Sale,
            quantity: -2,
            created_at: now,
            created_by: 1,
            description: None,
            balance_after: 0,
            reference_id: None,
        };

        let response: StockMovementResponse = row.into();

        assert_eq!(response.id, 2);
        assert_eq!(response.order_id, None);
        assert_eq!(response.product_id, 101);
        assert_eq!(response.product_name, "Another Product");
        assert_eq!(response.r#type, StockMovementType::Sale);
        assert_eq!(response.quantity, -2);
        assert_eq!(response.created_at, now);
        assert_eq!(response.created_by, 1);
        assert_eq!(response.description, None);
    }
}
