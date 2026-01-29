use rust_decimal::prelude::ToPrimitive;
use shared_dtos::user_subscription::UserSubscriptionBotResponse;

use crate::models::user_subscription::UserSubscriptionEnrichedRow;

impl From<UserSubscriptionEnrichedRow> for UserSubscriptionBotResponse {
    fn from(value: UserSubscriptionEnrichedRow) -> Self {
        UserSubscriptionBotResponse {
            id: value.id,
            customer_id: value.customer_id,
            product_id: value.product_id,
            product_name: value.product_name,
            cancelled_at: value.cancelled_at,
            details: value.details.and_then(|v| serde_json::from_value(v).ok()),
            expires_at: value.expires_at,
            next_charge_at: value.next_charge_at,
            order_id: value.order_id,
            period_days: value.period_days,
            price_at_subscription: value.price_at_subscription.to_f64().unwrap_or_default(),
            renewal_order_id: value.renewal_order_id,
            started_at: value.started_at,
        }
    }
}
