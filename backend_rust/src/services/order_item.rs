use std::sync::Arc;

use async_trait::async_trait;
use shared_dtos::stock_movement::StockMovementType;

use crate::{
    errors::api::ApiResult,
    infrastructure::repositories::{
        order_item::{OrderItemRepository, OrderItemRepositoryTrait},
        stock_movement::{StockMovementRepository, StockMovementRepositoryTrait},
    },
    models::{
        order_item::{NewOrderItem, OrderItemRow},
        stock_movement::NewStockMovement,
    },
};

#[async_trait]
pub trait OrderItemServiceTrait: Send + Sync {
    async fn get_for_order(&self, order_id: i64) -> ApiResult<Vec<OrderItemRow>>;
    async fn create(&self, order: NewOrderItem) -> ApiResult<OrderItemRow>;
}

pub struct OrderItemService<R, S> {
    order_repo: Arc<R>,
    stock_movement_repo: Arc<S>,
}

impl<R, S> OrderItemService<R, S>
where
    R: OrderItemRepositoryTrait + Send + Sync,
    S: StockMovementRepositoryTrait + Send + Sync,
{
    pub fn new(order_repo: Arc<R>, stock_movement_repo: Arc<S>) -> Self {
        Self {
            order_repo,
            stock_movement_repo,
        }
    }
}

#[async_trait]
impl OrderItemServiceTrait for OrderItemService<OrderItemRepository, StockMovementRepository> {
    async fn get_for_order(&self, order_id: i64) -> ApiResult<Vec<OrderItemRow>> {
        let res = self.order_repo.get_for_order(order_id).await?;
        Ok(res)
    }

    async fn create(&self, order_item: NewOrderItem) -> ApiResult<OrderItemRow> {
        let res = self.order_repo.create(order_item.clone()).await?;
        self.stock_movement_repo
            .create(NewStockMovement {
                description: None,
                order_id: Some(order_item.order_id),
                product_id: order_item.product_id,
                quantity: order_item.quantity as i64,
                reference_id: None,
                r#type: StockMovementType::Sale,
                created_by: 1, // System
            })
            .await?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::{
        order_item::OrderItemRepository, stock_movement::StockMovementRepository,
    };
    use rust_decimal::Decimal;
    use sqlx::PgPool;
    use std::sync::Arc;

    #[derive(Debug)]
    struct StockMovementCheck {
        r#type: StockMovementType,
        quantity: i64,
        created_by: i64,
    }

    async fn create_customer(pool: &PgPool, telegram_id: i64) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot) VALUES ($1, 1, 1) RETURNING id",
            telegram_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_bot(pool: &PgPool, owner_id: Option<i64>, token: &str, username: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO bots (
                owner_id, token, username, type, is_active, is_primary, referral_percentage, created_by
            )
            VALUES ($1, $2, $3, 'main', true, false, 0.0, 1)
            RETURNING id
            "#,
            owner_id,
            token,
            username
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_product(pool: &PgPool, name: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO products (name, base_price, type, created_by, provider_name)
            VALUES ($1, 10.0, 'item', 1, 'test')
            RETURNING id
            "#,
            name
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_order(pool: &PgPool, customer_id: i64, bot_id: i64) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO orders (customer_id, bot_id, status, amount, currency) VALUES ($1, $2, 'created', 10.0, 'USD') RETURNING id",
            customer_id,
            bot_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    fn build_service(
        pool: &PgPool,
    ) -> OrderItemService<OrderItemRepository, StockMovementRepository> {
        let pool = Arc::new(pool.clone());
        OrderItemService::new(
            Arc::new(OrderItemRepository::new(pool.clone())),
            Arc::new(StockMovementRepository::new(pool.clone())),
        )
    }

    #[sqlx::test]
    async fn test_create_order_item_creates_stock_movement(pool: PgPool) {
        let service = build_service(&pool);
        let customer_id = create_customer(&pool, 50505).await;
        let bot_id = create_bot(&pool, Some(customer_id), "order_item_bot", "order_item_bot").await;
        let product_id = create_product(&pool, "order_item_product").await;
        let order_id = create_order(&pool, customer_id, bot_id).await;

        let created = service
            .create(NewOrderItem {
                order_id,
                product_id,
                name_at_purchase: "Test Product".to_string(),
                price_at_purchase: Decimal::from(10),
                quantity: 2,
                fulfillment_type: "text".to_string(),
                fulfillment_content: None,
                fulfillment_image_id: None,
                details: None,
            })
            .await
            .unwrap();

        assert_eq!(created.order_id, order_id);
        assert_eq!(created.product_id, product_id);

        let movement = sqlx::query_as!(
            StockMovementCheck,
            r#"
            SELECT type as "type: _", quantity, created_by
            FROM stock_movements
            WHERE order_id = $1 AND product_id = $2
            ORDER BY id DESC
            LIMIT 1
            "#,
            order_id,
            product_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(movement.r#type, StockMovementType::Sale);
        assert_eq!(movement.quantity, 2);
        assert_eq!(movement.created_by, 1);
    }

    #[sqlx::test]
    async fn test_get_for_order_returns_items(pool: PgPool) {
        let service = build_service(&pool);
        let customer_id = create_customer(&pool, 60606).await;
        let bot_id = create_bot(
            &pool,
            Some(customer_id),
            "order_item_bot_2",
            "order_item_bot_2",
        )
        .await;
        let product_id = create_product(&pool, "order_item_product_2").await;
        let order_id = create_order(&pool, customer_id, bot_id).await;

        service
            .create(NewOrderItem {
                order_id,
                product_id,
                name_at_purchase: "Test Product".to_string(),
                price_at_purchase: Decimal::from(10),
                quantity: 1,
                fulfillment_type: "text".to_string(),
                fulfillment_content: None,
                fulfillment_image_id: None,
                details: None,
            })
            .await
            .unwrap();

        let items = service.get_for_order(order_id).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].order_id, order_id);
    }
}
