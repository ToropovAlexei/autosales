use std::sync::Arc;

use async_trait::async_trait;
use bigdecimal::BigDecimal;
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        products::{ProductRepository, ProductRepositoryTrait},
        stock_movement::{StockMovementRepository, StockMovementRepositoryTrait},
    },
    models::{
        common::PaginatedResult,
        product::{NewProduct, ProductListQuery, ProductRow, ProductType, UpdateProduct},
        stock_movement::{NewStockMovement, StockMovementType},
    },
};

#[derive(Debug)]
pub struct CreateProduct {
    pub name: String,
    pub price: BigDecimal,
    pub category_id: i64,
    pub image_id: Option<Uuid>,
    pub r#type: ProductType,
    pub subscription_period_days: Option<i16>,
    pub details: Option<serde_json::Value>,
    pub fulfillment_text: Option<String>,
    pub fulfillment_image_id: Option<Uuid>,
    pub provider_name: String,
    pub external_id: Option<String>,
    pub created_by: i64,
    pub initial_stock: Option<i64>,
}

#[derive(Debug)]
pub struct UpdateProductCommand {
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
    pub stock: Option<i64>,
}

#[async_trait]
pub trait ProductServiceTrait: Send + Sync {
    async fn get_list(&self, query: ProductListQuery) -> ApiResult<PaginatedResult<ProductRow>>;
    async fn create(&self, product: CreateProduct) -> ApiResult<ProductRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<ProductRow>;
    async fn update(&self, id: i64, product: UpdateProductCommand) -> ApiResult<ProductRow>;
    async fn delete(&self, id: i64) -> ApiResult<()>;
}

pub struct ProductService<R, S> {
    product_repo: Arc<R>,
    stock_movement_repo: Arc<S>,
}

impl<R, S> ProductService<R, S>
where
    R: ProductRepositoryTrait + Send + Sync,
    S: StockMovementRepositoryTrait + Send + Sync,
{
    pub fn new(product_repo: Arc<R>, stock_movement_repo: Arc<S>) -> Self {
        Self {
            product_repo,
            stock_movement_repo,
        }
    }
}

#[async_trait]
impl ProductServiceTrait for ProductService<ProductRepository, StockMovementRepository> {
    async fn get_list(&self, query: ProductListQuery) -> ApiResult<PaginatedResult<ProductRow>> {
        self.product_repo
            .get_list(query)
            .await
            .map_err(ApiError::from)
    }

    async fn create(&self, product: CreateProduct) -> ApiResult<ProductRow> {
        let created = self
            .product_repo
            .create(NewProduct {
                category_id: product.category_id,
                created_by: product.created_by,
                details: product.details,
                external_id: product.external_id,
                fulfillment_image_id: product.fulfillment_image_id,
                fulfillment_text: product.fulfillment_text,
                image_id: product.image_id,
                name: product.name,
                price: product.price,
                provider_name: product.provider_name,
                subscription_period_days: product.subscription_period_days.unwrap_or_default(),
                r#type: product.r#type,
            })
            .await?;

        if let Some(initial_stock) = product.initial_stock {
            self.stock_movement_repo
                .create(NewStockMovement {
                    product_id: created.id,
                    quantity: initial_stock,
                    created_by: product.created_by,
                    r#type: StockMovementType::Initial,
                    order_id: None,
                    description: None,
                    reference_id: None,
                })
                .await?;
        }

        Ok(created)
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<ProductRow> {
        let res = self.product_repo.get_by_id(id).await?;
        Ok(res)
    }

    async fn update(&self, id: i64, product: UpdateProductCommand) -> ApiResult<ProductRow> {
        let updated = self
            .product_repo
            .update(
                id,
                UpdateProduct {
                    category_id: product.category_id,
                    details: product.details,
                    external_id: product.external_id,
                    fulfillment_image_id: product.fulfillment_image_id,
                    fulfillment_text: product.fulfillment_text,
                    image_id: product.image_id,
                    name: product.name,
                    price: product.price,
                    subscription_period_days: product.subscription_period_days,
                    r#type: product.r#type,
                },
            )
            .await?;

        if let Some(stock) = product.stock {
            let current = self
                .stock_movement_repo
                .get_last_by_product_id(updated.id)
                .await
                .map(|r| r.quantity)
                // If initial stock is zero, we don't have any stock movement
                .unwrap_or_default();

            if current == stock {
                return Ok(updated);
            }
            self.stock_movement_repo
                .create(NewStockMovement {
                    product_id: updated.id,
                    quantity: stock - current,
                    created_by: updated.created_by,
                    r#type: StockMovementType::Adjustment,
                    order_id: None,
                    description: None,
                    reference_id: None,
                })
                .await?;
        }

        Ok(updated)
    }

    async fn delete(&self, id: i64) -> ApiResult<()> {
        Ok(self.product_repo.delete(id).await?)
    }
}
