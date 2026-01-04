use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::define_list_query;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS, ToSchema)]
#[sqlx(type_name = "product_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "product.ts")]
pub enum ProductType {
    Item,
    Subscription,
}

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct ProductRow {
    pub id: i64,
    pub name: String,
    pub price: BigDecimal,
    pub category_id: Option<i64>,
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

#[derive(Debug)]
pub struct NewProduct {
    pub name: String,
    pub price: BigDecimal,
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
    pub price: Option<BigDecimal>,
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
            Price => "price",
        ]
    },
    order_fields: {
        ProductOrderFields,
        [
            Id => "id",
            Name => "name",
            CategoryId => "category_id",
            Price => "price",
            ProviderName => "provider_name",
            CreatedAt => "created_at",
        ]
    }
}
