use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use validator::Validate;

use crate::models::customer::CustomerRow;

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "customer.ts", rename = "Customer")]
pub struct CustomerResponse {
    pub id: i64,
    pub telegram_id: i64,
    pub balance: f64,
    pub is_blocked: bool,
    pub bot_is_blocked_by_user: bool,
    pub has_passed_captcha: bool,
    pub registered_with_bot: i64,
    pub last_seen_with_bot: i64,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CustomerRow> for CustomerResponse {
    fn from(r: CustomerRow) -> Self {
        CustomerResponse {
            id: r.id,
            telegram_id: r.telegram_id,
            balance: r.balance.to_f64().unwrap_or_default(),
            is_blocked: r.is_blocked,
            bot_is_blocked_by_user: r.bot_is_blocked_by_user,
            has_passed_captcha: r.has_passed_captcha,
            registered_with_bot: r.registered_with_bot,
            last_seen_with_bot: r.last_seen_with_bot,
            last_seen_at: r.last_seen_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "customer.ts", rename = "UpdateCustomer")]
pub struct UpdateCustomerRequest {
    #[ts(optional)]
    pub is_blocked: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use validator::Validate;

    #[test]
    fn test_customer_response_from_customer_row() {
        let now = Utc::now();
        let customer_row = CustomerRow {
            id: 1,
            telegram_id: 12345,
            balance: Decimal::new(10050, 2), // 100.50
            is_blocked: false,
            bot_is_blocked_by_user: false,
            has_passed_captcha: true,
            registered_with_bot: 1,
            last_seen_with_bot: 1,
            last_seen_at: now,
            created_at: now,
            updated_at: now,
        };

        let customer_response: CustomerResponse = customer_row.into();

        assert_eq!(customer_response.id, 1);
        assert_eq!(customer_response.telegram_id, 12345);
        assert_eq!(customer_response.balance, 100.50);
        assert!(!customer_response.is_blocked);
        assert!(!customer_response.bot_is_blocked_by_user);
        assert!(customer_response.has_passed_captcha);
        assert_eq!(customer_response.registered_with_bot, 1);
        assert_eq!(customer_response.last_seen_with_bot, 1);
        assert_eq!(customer_response.last_seen_at, now);
        assert_eq!(customer_response.created_at, now);
        assert_eq!(customer_response.updated_at, now);
    }

    #[test]
    fn test_update_customer_request_validation() {
        // Valid: is_blocked is Some(true)
        let req = UpdateCustomerRequest {
            is_blocked: Some(true),
        };
        assert!(req.validate().is_ok());

        // Valid: is_blocked is Some(false)
        let req = UpdateCustomerRequest {
            is_blocked: Some(false),
        };
        assert!(req.validate().is_ok());

        // Valid: is_blocked is None
        let req = UpdateCustomerRequest { is_blocked: None };
        assert!(req.validate().is_ok());
    }
}
