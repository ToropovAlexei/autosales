use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::define_list_query;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS, ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "bot.ts")]
pub enum BotType {
    Main,
    Referral,
}

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct BotRow {
    pub id: i64,
    pub owner_id: Option<i64>,
    pub token: String,
    pub username: String,
    pub r#type: BotType,
    pub is_active: bool,
    pub is_primary: bool,
    pub referral_percentage: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i64>,
}

#[derive(Debug)]
pub struct NewBot {
    pub owner_id: Option<i64>,
    pub token: String,
    pub username: String,
    pub r#type: BotType,
    pub is_active: bool,
    pub is_primary: bool,
    pub referral_percentage: BigDecimal,
    pub created_by: Option<i64>,
}

#[derive(Debug)]
pub struct UpdateBot {
    pub username: Option<String>,
    pub is_active: Option<bool>,
    pub is_primary: Option<bool>,
    pub referral_percentage: Option<BigDecimal>,
}

define_list_query! {
    query_name: BotListQuery,
    filter_fields: {
        BotFilterFields,
        [
            Id => "id",
            Username => "username",
            Type => "type",
            IsPrimary => "is_primary",
            IsActive => "is_active",
            ReferralPercentage => "referral_percentage",
            CreatedAt => "created_at",
        ]
    },
    order_fields: {
        BotOrderFields,
        [
            Id => "id",
            Username => "username",
            Type => "type",
            IsPrimary => "is_primary",
            IsActive => "is_active",
            ReferralPercentage => "referral_percentage",
            CreatedAt => "created_at",
        ]
    }
}
