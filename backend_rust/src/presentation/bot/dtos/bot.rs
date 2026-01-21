use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};
use validator::Validate;

use crate::models::bot::BotRow;
use rust_decimal::prelude::ToPrimitive;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct BotResponse {
    pub id: i64,
    pub token: String,
    pub username: String,
    pub is_active: bool,
    pub is_primary: bool,
    pub referral_percentage: f64,
    pub owner_id: Option<i64>,
}

impl From<BotRow> for BotResponse {
    fn from(r: BotRow) -> Self {
        BotResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, ToResponse)]
pub struct NewBotRequest {
    #[validate(length(min = 44, max = 48, message = "Length must be between 44 and 48"))]
    pub token: String,
    pub owner_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, ToResponse)]
pub struct UpdateBotRequest {
    pub is_active: Option<bool>,
    pub is_primary: Option<bool>,
}
