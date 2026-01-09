use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::define_list_query;

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct UserSubscriptionRow {
    pub id: i64,
    pub customer_id: i64,
    pub product_id: Option<i64>,
    pub order_id: i64,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub next_charge_at: Option<DateTime<Utc>>,
    pub renewal_order_id: Option<i64>,
    pub price_at_subscription: Decimal,
    pub period_days: i16,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct UserSubscriptionEnrichedRow {
    pub id: i64,
    pub customer_id: i64,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub order_id: i64,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub next_charge_at: Option<DateTime<Utc>>,
    pub renewal_order_id: Option<i64>,
    pub price_at_subscription: Decimal,
    pub period_days: i16,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewUserSubscription {
    pub customer_id: i64,
    pub product_id: Option<i64>,
    pub order_id: i64,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub next_charge_at: Option<DateTime<Utc>>,
    pub price_at_subscription: Decimal,
    pub period_days: i16,
    pub details: Option<serde_json::Value>,
}

define_list_query! {
    query_name: UserSubscriptionListQuery,
    filter_fields: {
        UserSubscriptionFilterFields,
        [
            Id => "id",
        ]
    },
    order_fields: {
        UserSubscriptionOrderFields,
        [
            Id => "id",
        ]
    }
}
