use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
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
