use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerBotResponse {
    pub id: i64,
    pub telegram_id: i64,
    pub balance: f64,
    pub is_blocked: bool,
    pub bot_is_blocked_by_user: bool,
    pub has_passed_captcha: bool,
    pub blocked_until: Option<DateTime<Utc>>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCustomerBotRequest {
    pub telegram_id: i64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCustomerBotRequest {
    pub bot_is_blocked_by_user: Option<bool>,
    pub has_passed_captcha: Option<bool>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "customer.ts", rename = "Customer")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAdminResponse {
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

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "customer.ts", rename = "UpdateCustomer")
)]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCustomerAdminRequest {
    #[cfg_attr(feature = "ts", ts(optional))]
    pub is_blocked: Option<bool>,
}
