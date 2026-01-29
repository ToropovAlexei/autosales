use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use shared_dtos::customer::CustomerBotResponse;
use utoipa::{ToResponse, ToSchema};
use validator::Validate;

use crate::models::customer::CustomerRow;

impl From<CustomerRow> for CustomerBotResponse {
    fn from(r: CustomerRow) -> Self {
        CustomerBotResponse {
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
#[schema(as = bot::NewCustomerRequest)]
pub struct NewCustomerRequest {
    pub telegram_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, ToResponse)]
#[schema(as = bot::UpdateCustomerRequest)]
pub struct UpdateCustomerRequest {
    pub bot_is_blocked_by_user: Option<bool>,
    pub has_passed_captcha: Option<bool>,
}
