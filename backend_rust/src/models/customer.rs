use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::define_list_query;

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct CustomerRow {
    pub id: i64,
    pub telegram_id: i64,
    pub balance: Decimal,
    pub is_blocked: bool,
    pub blocked_until: Option<DateTime<Utc>>,
    pub bot_is_blocked_by_user: bool,
    pub has_passed_captcha: bool,
    pub registered_with_bot: i64,
    pub last_seen_with_bot: i64,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewCustomer {
    pub telegram_id: i64,
    pub registered_with_bot: i64,
}

#[derive(Debug, Default)]
pub struct UpdateCustomer {
    pub is_blocked: Option<bool>,
    pub bot_is_blocked_by_user: Option<bool>,
    pub has_passed_captcha: Option<bool>,
    pub last_seen_with_bot: Option<i64>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub blocked_until: Option<Option<DateTime<Utc>>>,
}

define_list_query! {
    query_name: CustomerListQuery,
    filter_fields: {
        CustomerFilterFields,
        [
            Id => "id",
            TelegramId => "telegram_id",
            Balance => "balance",
            IsBlocked => "is_blocked",
            BotIsBlockedByUser => "bot_is_blocked_by_user",
            HasPassedCaptcha => "has_passed_captcha",
            RegisteredWithBot => "registered_with_bot",
            LastSeenWithBot => "last_seen_with_bot",
            LastSeenAt => "last_seen_at",
            CreatedAt => "created_at",
            UpdatedAt => "updated_at",
        ]
    },
    order_fields: {
        CustomerOrderFields,
        [
            Id => "id",
            TelegramId => "telegram_id",
            Balance => "balance",
            IsBlocked => "is_blocked",
            BotIsBlockedByUser => "bot_is_blocked_by_user",
            HasPassedCaptcha => "has_passed_captcha",
            RegisteredWithBot => "registered_with_bot",
            LastSeenWithBot => "last_seen_with_bot",
            LastSeenAt => "last_seen_at",
            CreatedAt => "created_at",
            UpdatedAt => "updated_at",
        ]
    }
}
