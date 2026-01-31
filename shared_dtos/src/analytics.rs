use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Serialize, Deserialize)]
pub struct BotAnalyticsBotResponse {
    pub bot_id: i64,
    pub total_earnings: f64,
    pub purchase_count: i64,
}
