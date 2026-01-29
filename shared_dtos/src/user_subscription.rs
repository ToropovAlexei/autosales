use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSubscriptionBotResponse {
    pub id: i64,
    pub customer_id: i64,
    pub product_id: Option<i64>,
    pub order_id: i64,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub next_charge_at: Option<DateTime<Utc>>,
    pub renewal_order_id: Option<i64>,
    pub price_at_subscription: f64,
    pub period_days: i16,
    pub details: Option<UserSubscriptionDetails>,
    pub product_name: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserSubscriptionDetails {
    ContMs {
        host: String,
        port: u16,
        username: String,
        password: String,
    },
}
