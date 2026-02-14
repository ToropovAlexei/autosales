use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "bot.ts"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotType {
    Main,
    Referral,
}

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

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "bot.ts", rename = "Bot"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotAdminResponse {
    pub id: i64,
    pub owner_id: Option<i64>,
    pub token: String,
    pub username: String,
    pub r#type: BotType,
    pub is_active: bool,
    pub is_primary: bool,
    pub referral_percentage: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i64>,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "bot.ts", rename = "NewBot"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBotAdminRequest {
    #[cfg_attr(
        feature = "validate",
        validate(length(min = 44, max = 48, message = "Length must be between 44 and 48"))
    )]
    pub token: String,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "bot.ts", rename = "UpdateBot"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "validate", derive(validator::Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBotAdminRequest {
    #[cfg_attr(feature = "ts", ts(optional))]
    pub is_active: Option<bool>,
    #[cfg_attr(feature = "ts", ts(optional))]
    pub is_primary: Option<bool>,
    #[cfg_attr(feature = "ts", ts(optional))]
    #[cfg_attr(feature = "validate", validate(range(min = 0.0, max = 100.0)))]
    pub referral_percentage: Option<f64>,
}
