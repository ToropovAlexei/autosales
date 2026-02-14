use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared_dtos::bot::BotType;
use sqlx::prelude::FromRow;

use crate::define_list_query;

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct BotRow {
    pub id: i64,
    pub owner_id: Option<i64>,
    pub token: String,
    pub username: String,
    pub r#type: BotType,
    pub is_active: bool,
    pub is_primary: bool,
    pub referral_percentage: Decimal,
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
    pub referral_percentage: Decimal,
    pub created_by: Option<i64>,
}

#[derive(Debug)]
pub struct UpdateBot {
    pub username: Option<String>,
    pub is_active: Option<bool>,
    pub is_primary: Option<bool>,
    pub referral_percentage: Option<Decimal>,
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
            OwnerId => "owner_id",
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
            OwnerId => "owner_id",
        ]
    }
}
