use shared_dtos::analytics::BotAnalyticsBotResponse;

use crate::models::analytics::BotAnalyticsRow;
use rust_decimal::prelude::ToPrimitive;

impl From<BotAnalyticsRow> for BotAnalyticsBotResponse {
    fn from(r: BotAnalyticsRow) -> Self {
        BotAnalyticsBotResponse {
            bot_id: r.bot_id,
            purchase_count: r.purchase_count,
            total_earnings: r.total_earnings.to_f64().unwrap_or_default(),
        }
    }
}
