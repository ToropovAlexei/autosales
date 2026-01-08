use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToResponse, ToSchema};
use validator::Validate;

use crate::models::bot::{BotRow, BotType};
use bigdecimal::ToPrimitive;

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "bot.ts", rename = "Bot")]
pub struct BotResponse {
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

impl From<BotRow> for BotResponse {
    fn from(r: BotRow) -> Self {
        BotResponse {
            id: r.id,
            owner_id: r.owner_id,
            token: r.token,
            username: r.username,
            r#type: r.r#type,
            is_active: r.is_active,
            is_primary: r.is_primary,
            referral_percentage: r.referral_percentage.to_f64().unwrap_or_default(),
            created_at: r.created_at,
            updated_at: r.updated_at,
            created_by: r.created_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "bot.ts", rename = "NewBot")]
pub struct NewBotRequest {
    #[validate(length(min = 44, max = 48, message = "Length must be between 44 and 48"))]
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, ToSchema, ToResponse)]
#[ts(export, export_to = "bot.ts", rename = "UpdateBot")]
pub struct UpdateBotRequest {
    #[ts(optional)]
    pub is_active: Option<bool>,
    #[ts(optional)]
    pub is_primary: Option<bool>,
    #[ts(optional)]
    #[validate(range(min = 0.0, max = 100.0))]
    pub referral_percentage: Option<f64>,
}
