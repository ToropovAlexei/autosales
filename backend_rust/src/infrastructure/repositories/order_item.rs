use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    errors::repository::RepositoryResult,
    models::order_item::{NewOrderItem, OrderItemRow},
};

#[async_trait]
pub trait OrderItemRepositoryTrait {
    async fn create(&self, order_item: NewOrderItem) -> RepositoryResult<OrderItemRow>;
    async fn get_for_order(&self, id: i64) -> RepositoryResult<Vec<OrderItemRow>>;
}

#[derive(Clone)]
pub struct OrderItemRepository {
    pool: Arc<PgPool>,
}

impl OrderItemRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrderItemRepositoryTrait for OrderItemRepository {
    async fn create(&self, order_item: NewOrderItem) -> RepositoryResult<OrderItemRow> {
        let result = sqlx::query_as!(
            OrderItemRow,
            r#"
            INSERT INTO order_items (
                order_id, product_id, name_at_purchase, price_at_purchase, quantity,
                fulfillment_type, fulfillment_content, fulfillment_image_id, details
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            order_item.order_id,
            order_item.product_id,
            order_item.name_at_purchase,
            order_item.price_at_purchase,
            order_item.quantity,
            order_item.fulfillment_type,
            order_item.fulfillment_content,
            order_item.fulfillment_image_id,
            order_item.details
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn get_for_order(&self, id: i64) -> RepositoryResult<Vec<OrderItemRow>> {
        let result = sqlx::query_as!(
            OrderItemRow,
            "SELECT * FROM order_items WHERE order_id = $1",
            id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(result)
    }
}
