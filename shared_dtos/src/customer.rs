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
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCustomerBotRequest {
    pub telegram_id: i64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCustomerBotRequest {
    pub bot_is_blocked_by_user: Option<bool>,
    pub has_passed_captcha: Option<bool>,
}
