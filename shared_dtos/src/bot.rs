use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotBotResponse {
    pub id: i64,
    pub token: String,
    pub username: String,
    pub is_active: bool,
    pub is_primary: bool,
    pub referral_percentage: f64,
    pub owner_id: Option<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBotBotRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(min = 44, max = 48, message = "Length must be between 44 and 48"))
    )]
    pub token: String,
    pub owner_id: i64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateBotBotRequest {
    pub is_active: Option<bool>,
    pub is_primary: Option<bool>,
}
