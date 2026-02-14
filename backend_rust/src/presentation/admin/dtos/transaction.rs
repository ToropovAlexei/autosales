use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use shared_dtos::invoice::PaymentSystem;
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};

use crate::models::transaction::{TransactionRow, TransactionType};

#[derive(Debug, Serialize, Deserialize, ToSchema, ToResponse, TS)]
#[ts(export, export_to = "transaction.ts", rename = "Transaction")]
pub struct TransactionAdminResponse {
    pub id: i64,
    pub customer_id: Option<i64>,
    pub order_id: Option<i64>,
    pub r#type: TransactionType,
    pub amount: f64,
    pub store_balance_delta: f64,
    pub platform_commission: f64,
    pub gateway_commission: f64,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub payment_gateway: Option<PaymentSystem>,
}

impl From<TransactionRow> for TransactionAdminResponse {
    fn from(row: TransactionRow) -> Self {
        Self {
            id: row.id,
            customer_id: row.customer_id,
            order_id: row.order_id,
            r#type: row.r#type,
            amount: row.amount.to_f64().unwrap_or_default(),
            store_balance_delta: row.store_balance_delta.to_f64().unwrap_or_default(),
            platform_commission: row.platform_commission.to_f64().unwrap_or_default(),
            gateway_commission: row.gateway_commission.to_f64().unwrap_or_default(),
            created_at: row.created_at,
            description: row.description,
            payment_gateway: row.payment_gateway,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::{TransactionRow, TransactionType};
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal_macros::dec;

    #[test]
    fn test_transaction_response_from_transaction_row() {
        let now = Utc::now();
        let transaction_row = TransactionRow {
            id: 1,
            customer_id: Some(101),
            order_id: Some(202),
            r#type: TransactionType::Purchase,
            amount: Decimal::from_f64(100.50).unwrap(),
            store_balance_delta: Decimal::from_f64(-5.00).unwrap(),
            platform_commission: Decimal::from_f64(2.50).unwrap(),
            gateway_commission: Decimal::from_f64(1.00).unwrap(),
            created_at: now,
            description: Some("Test transaction".to_string()),
            payment_gateway: Some(PaymentSystem::Mock),
            user_balance_after: None,
            store_balance_after: dec!(0),
            details: None,
            bot_id: None,
        };

        let transaction_response: TransactionAdminResponse = transaction_row.into();

        assert_eq!(transaction_response.id, 1);
        assert_eq!(transaction_response.customer_id, Some(101));
        assert_eq!(transaction_response.order_id, Some(202));
        assert_eq!(transaction_response.r#type, TransactionType::Purchase);
        assert_eq!(transaction_response.amount, 100.50);
        assert_eq!(transaction_response.store_balance_delta, -5.00);
        assert_eq!(transaction_response.platform_commission, 2.50);
        assert_eq!(transaction_response.gateway_commission, 1.00);
        assert_eq!(transaction_response.created_at, now);
        assert_eq!(
            transaction_response.description,
            Some("Test transaction".to_string())
        );
        assert_eq!(
            transaction_response.payment_gateway,
            Some(PaymentSystem::Mock)
        );
    }

    #[test]
    fn test_transaction_response_from_transaction_row_defaults() {
        let now = Utc::now();
        let transaction_row = TransactionRow {
            id: 2,
            customer_id: None,
            order_id: None,
            r#type: TransactionType::Refund,
            amount: Decimal::from_f64(0.0).unwrap(),
            store_balance_delta: Decimal::from_f64(0.0).unwrap(),
            platform_commission: Decimal::from_f64(0.0).unwrap(),
            gateway_commission: Decimal::from_f64(0.0).unwrap(),
            created_at: now,
            description: None,
            payment_gateway: None,
            user_balance_after: None,
            store_balance_after: dec!(0),
            details: None,
            bot_id: None,
        };

        let transaction_response: TransactionAdminResponse = transaction_row.into();

        assert_eq!(transaction_response.id, 2);
        assert_eq!(transaction_response.customer_id, None);
        assert_eq!(transaction_response.order_id, None);
        assert_eq!(transaction_response.r#type, TransactionType::Refund);
        assert_eq!(transaction_response.amount, 0.0);
        assert_eq!(transaction_response.store_balance_delta, 0.0);
        assert_eq!(transaction_response.platform_commission, 0.0);
        assert_eq!(transaction_response.gateway_commission, 0.0);
        assert_eq!(transaction_response.created_at, now);
        assert_eq!(transaction_response.description, None);
        assert_eq!(transaction_response.payment_gateway, None);
    }
}
