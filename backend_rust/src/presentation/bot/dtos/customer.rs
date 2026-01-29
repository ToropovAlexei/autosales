use rust_decimal::prelude::ToPrimitive;
use shared_dtos::customer::CustomerBotResponse;

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
