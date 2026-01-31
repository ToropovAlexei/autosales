use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared_dtos::product::ProductType;
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::define_list_query;

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct ProductRow {
    pub id: i64,
    pub name: String,
    pub base_price: Decimal,
    pub category_id: Option<i64>,
    pub stock: i32,
    pub image_id: Option<Uuid>,
    pub r#type: ProductType,
    pub subscription_period_days: i16,
    pub details: Option<serde_json::Value>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub fulfillment_text: Option<String>,
    pub fulfillment_image_id: Option<Uuid>,
    pub provider_name: String,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: i64,
}

#[derive(Debug, Clone)]
pub struct NewProduct {
    pub name: String,
    pub base_price: Decimal,
    pub category_id: i64,
    pub image_id: Option<Uuid>,
    pub r#type: ProductType,
    pub subscription_period_days: i16,
    pub details: Option<serde_json::Value>,
    pub fulfillment_text: Option<String>,
    pub fulfillment_image_id: Option<Uuid>,
    pub provider_name: String,
    pub external_id: Option<String>,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct UpdateProduct {
    pub name: Option<String>,
    pub base_price: Option<Decimal>,
    pub category_id: Option<i64>,
    pub image_id: Option<Option<Uuid>>,
    pub r#type: Option<ProductType>,
    pub subscription_period_days: Option<i16>,
    pub details: Option<Option<serde_json::Value>>,
    pub fulfillment_text: Option<Option<String>>,
    pub fulfillment_image_id: Option<Option<Uuid>>,
    pub external_id: Option<Option<String>>,
}

define_list_query! {
    query_name: ProductListQuery,
    filter_fields: {
        ProductFilterFields,
        [
            Name => "name",
            CategoryId => "category_id",
            ProviderName => "provider_name",
            ExternalId => "external_id",
            BasePrice => "base_price",
            Type => "type",
        ]
    },
    order_fields: {
        ProductOrderFields,
        [
            Id => "id",
            Name => "name",
            CategoryId => "category_id",
            BasePrice => "base_price",
            ProviderName => "provider_name",
            CreatedAt => "created_at",
            Type => "type",
        ]
    }
}
