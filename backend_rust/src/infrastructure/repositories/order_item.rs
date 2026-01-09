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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use sqlx::PgPool;

    async fn create_test_user(pool: &PgPool, login: &str) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO admin_users (login, hashed_password, two_fa_secret, created_by) VALUES ($1, 'password', '', 1) RETURNING id",
            login
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_bot(pool: &PgPool, token: &str, username: &str) -> i64 {
        sqlx::query_scalar!(
            r#"INSERT INTO bots (owner_id, token, username, type, is_active, is_primary, referral_percentage, created_by) VALUES (1, $1, $2, 'main', true, false, 0.1, 1) RETURNING id"#,
            token,
            username
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_customer(pool: &PgPool, telegram_id: i64) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot) VALUES ($1, 1, 1) RETURNING id",
            telegram_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_product(pool: &PgPool, name: &str) -> i64 {
        sqlx::query_scalar!(
            r#"INSERT INTO products (name, base_price, type, created_by, provider_name) VALUES ($1, 10.0, 'item', 1, 'test') RETURNING id"#,
            name
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_order(pool: &PgPool, customer_id: i64, bot_id: i64) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO orders (customer_id, bot_id, status, amount, currency) VALUES ($1, $2, 'created', 10.0, 'USD') RETURNING id",
            customer_id,
            bot_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_get_order_item(pool: PgPool) {
        let repo = OrderItemRepository::new(Arc::new(pool.clone()));
        let _user_id = create_test_user(&pool, "order_item_user_1").await;
        let bot_id = create_test_bot(&pool, "order_item_bot_1", "order_item_bot_1").await;
        let customer_id = create_test_customer(&pool, 12345).await;
        let product_id = create_test_product(&pool, "order_item_product_1").await;
        let order_id = create_test_order(&pool, customer_id, bot_id).await;

        let new_order_item = NewOrderItem {
            order_id,
            product_id,
            name_at_purchase: "Test Product".to_string(),
            price_at_purchase: Decimal::from(10),
            quantity: 1,
            fulfillment_type: "text".to_string(),
            fulfillment_content: None,
            fulfillment_image_id: None,
            details: None,
        };

        // Create an order item
        let created_item = repo.create(new_order_item).await.unwrap();
        assert_eq!(created_item.order_id, order_id);

        // Get the order items for the order
        let items = repo.get_for_order(order_id).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, created_item.id);
    }
}
