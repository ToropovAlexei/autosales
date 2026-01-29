use shared_dtos::bot::BotBotResponse;

use crate::models::bot::BotRow;
use rust_decimal::prelude::ToPrimitive;

impl From<BotRow> for BotBotResponse {
    fn from(r: BotRow) -> Self {
        BotBotResponse {
            id: r.id,
            token: r.token,
            username: r.username,
            is_active: r.is_active,
            is_primary: r.is_primary,
            referral_percentage: r.referral_percentage.to_f64().unwrap_or_default(),
            owner_id: r.owner_id,
        }
    }
}
