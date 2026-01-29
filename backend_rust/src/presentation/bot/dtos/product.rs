use rust_decimal::prelude::ToPrimitive;
use shared_dtos::product::ProductBotResponse;

use crate::services::product::Product;

impl From<Product> for ProductBotResponse {
    fn from(r: Product) -> Self {
        ProductBotResponse {
            id: r.id,
            name: r.name,
            price: r.price.to_f64().unwrap_or_default(),
            category_id: r.category_id,
            image_id: r.image_id,
            r#type: r.r#type,
            subscription_period_days: r.subscription_period_days,
            details: r.details,
            fulfillment_text: r.fulfillment_text,
            fulfillment_image_id: r.fulfillment_image_id,
        }
    }
}
