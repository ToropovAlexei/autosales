use bigdecimal::ToPrimitive;
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};
use validator::Validate;

use crate::models::customer::CustomerRow;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct CustomerResponse {
    pub id: i64,
    pub telegram_id: i64,
    pub balance: f64,
    pub is_blocked: bool,
    pub bot_is_blocked_by_user: bool,
    pub has_passed_captcha: bool,
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, ToResponse)]
pub struct NewCustomerRequest {
    pub telegram_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, ToResponse)]
pub struct UpdateCustomerRequest {
    pub bot_is_blocked_by_user: Option<bool>,
    pub has_passed_captcha: Option<bool>,
}
