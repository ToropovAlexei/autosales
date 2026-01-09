use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder};

use crate::{
    errors::repository::RepositoryResult,
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        stock_movement::{NewStockMovement, StockMovementListQuery, StockMovementRow},
    },
};

#[async_trait]
pub trait StockMovementRepositoryTrait {
    async fn get_list(
        &self,
        query: StockMovementListQuery,
    ) -> RepositoryResult<PaginatedResult<StockMovementRow>>;
    async fn create(&self, stock_movement: NewStockMovement) -> RepositoryResult<StockMovementRow>;
    async fn get_last_by_product_id(&self, product_id: i64) -> RepositoryResult<StockMovementRow>;
}

#[derive(Clone)]
pub struct StockMovementRepository {
    pool: Arc<PgPool>,
}

impl StockMovementRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StockMovementRepositoryTrait for StockMovementRepository {
    async fn get_list(
        &self,
        query: StockMovementListQuery,
    ) -> RepositoryResult<PaginatedResult<StockMovementRow>> {
        let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM stock_movements");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder = QueryBuilder::new(
            r#"
        SELECT
            sm.id,
            sm.order_id,
            sm.product_id,
            sm.type,
            sm.quantity,
            sm.created_by,
            sm.description,
            sm.reference_id,
            sm.balance_after,
            sm.created_at,
            p.name AS product_name
        FROM stock_movements sm
        LEFT JOIN products p ON sm.product_id = p.id
        "#,
        );
        apply_list_query(&mut query_builder, &query);
        let query = query_builder.build_query_as::<StockMovementRow>();
        let items = query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn create(&self, stock_movement: NewStockMovement) -> RepositoryResult<StockMovementRow> {
        let inserted = sqlx::query_scalar!(
            r#"
            INSERT INTO stock_movements (order_id, product_id, type, quantity, created_by, description, reference_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
            stock_movement.order_id,
            stock_movement.product_id,
            stock_movement.r#type as _,
            stock_movement.quantity,
            stock_movement.created_by,
            stock_movement.description,
            stock_movement.reference_id
        )
        .fetch_one(&*self.pool)
        .await?;

        let row = sqlx::query_as!(
            StockMovementRow,
            r#"
            SELECT
                sm.id,
                sm.order_id,
                sm.product_id,
                sm.type as "type: _",
                sm.quantity,
                sm.created_by,
                sm.description,
                sm.reference_id,
                sm.balance_after,
                sm.created_at,
                p.name AS product_name
            FROM stock_movements sm
            LEFT JOIN products p ON sm.product_id = p.id
            WHERE sm.id = $1
            "#,
            inserted
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    async fn get_last_by_product_id(&self, product_id: i64) -> RepositoryResult<StockMovementRow> {
        let result = sqlx::query_as!(
            StockMovementRow,
            r#"
            SELECT
                sm.id,
                sm.order_id,
                sm.product_id,
                sm.type as "type: _",
                sm.quantity,
                sm.created_by,
                sm.description,
                sm.reference_id,
                sm.balance_after,
                sm.created_at,
                p.name AS product_name
            FROM stock_movements sm
            LEFT JOIN products p ON sm.product_id = p.id
            WHERE sm.product_id = $1
            ORDER BY sm.created_at DESC
            LIMIT 1
            "#,
            product_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use crate::models::admin_user::{AdminUserRow, NewAdminUser};
    use crate::models::bot::BotRow;
    use crate::models::customer::CustomerRow;
    use crate::models::order::OrderRow;
    use crate::models::product::ProductRow;
    use crate::models::stock_movement::StockMovementType;

    use super::*;

    async fn create_test_user(pool: &PgPool, login: &str) -> AdminUserRow {
        let new_user = NewAdminUser {
            login: login.to_string(),
            hashed_password: "password".to_string(),
            two_fa_secret: "".to_string(),
            telegram_id: None,
            created_by: 1,
        };
        let user_id: i64 = sqlx::query!(
            r#"
            INSERT INTO admin_users (login, hashed_password, two_fa_secret, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            new_user.login,
            new_user.hashed_password,
            new_user.two_fa_secret,
            new_user.created_by
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id;

        sqlx::query_as!(
            AdminUserRow,
            "SELECT * FROM admin_users WHERE id = $1",
            user_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_product(pool: &PgPool, name: &str, created_by: i64) -> ProductRow {
        let product_id: i64 = sqlx::query!(
            r#"
            INSERT INTO products (
                name, base_price, created_by, type, subscription_period_days,
                provider_name
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            name,
            Decimal::from(100), // price
            created_by,         // created_by
            "item",             // type - explicitly use string literal
            0,                  // subscription_period_days
            "test_provider"     // provider_name
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id;

        sqlx::query_as!(
            ProductRow,
            r#"SELECT 
                id, name, base_price, category_id, image_id, type as "type: _",
                subscription_period_days, details, deleted_at, fulfillment_text, 
                fulfillment_image_id, provider_name, external_id, created_at, 
                updated_at, created_by, stock
            FROM products WHERE id = $1"#,
            product_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_order(pool: &PgPool, customer_id: i64, bot_id: i64) -> OrderRow {
        let order_id: i64 = sqlx::query!(
            r#"
            INSERT INTO orders (customer_id, amount, currency, status, bot_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            customer_id,
            Decimal::from(100), // amount
            "USD",              // currency
            "created",          // status - explicitly use string literal
            bot_id              // bot_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id;

        sqlx::query_as!(
            OrderRow,
            r#"SELECT 
                id, customer_id, amount, currency, status as "status: _", bot_id, 
                created_at, updated_at, paid_at, fulfilled_at, cancelled_at
            FROM orders WHERE id = $1"#,
            order_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_customer(pool: &PgPool, telegram_id: i64, bot_id: i64) -> CustomerRow {
        sqlx::query_as!(
            CustomerRow,
            r#"
            INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            telegram_id,
            bot_id,
            bot_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_test_bot(pool: &PgPool) -> BotRow {
        let bot_id: i64 = sqlx::query!(
            r#"
            INSERT INTO bots (owner_id, token, username, type, is_active, is_primary, referral_percentage)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
            None as Option<i64>, // owner_id
            "test_token_stock".to_string(), // Ensure unique token
            "test_bot_stock".to_string(), // Ensure unique username
            "main", // type - explicitly use string literal
            true,            // is_active
            true,            // is_primary
            Decimal::from(0), // referral_percentage
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id; // Get the ID of the newly inserted bot

        // Now, fetch the full BotRow using query_as!
        sqlx::query_as!(
            BotRow,
            r#"SELECT 
                id, owner_id, token, username, type as "type: _", is_active, is_primary, 
                referral_percentage, created_at, updated_at, created_by
            FROM bots WHERE id = $1"#,
            bot_id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_balance_after_trigger(pool: PgPool) {
        let admin_user = create_test_user(&pool, "stock_admin").await;
        let product = create_test_product(&pool, "test_product_stock", admin_user.id).await;
        let bot = create_test_bot(&pool).await;
        let customer = create_test_customer(&pool, 12345, bot.id).await;
        let order = create_test_order(&pool, customer.id, bot.id).await;

        let repo = StockMovementRepository::new(Arc::new(pool.clone()));

        // Initial movement
        let initial_movement = NewStockMovement {
            order_id: None,
            product_id: product.id,
            r#type: StockMovementType::Initial,
            quantity: 100,
            created_by: admin_user.id,
            description: Some("Initial stock".to_string()),
            reference_id: None,
        };
        let result1 = repo.create(initial_movement).await.unwrap();
        assert_eq!(result1.balance_after, 100);

        // Restock movement
        let restock_movement = NewStockMovement {
            order_id: None,
            product_id: product.id,
            r#type: StockMovementType::Restock,
            quantity: 50,
            created_by: admin_user.id,
            description: Some("Restock".to_string()),
            reference_id: None,
        };
        let result2 = repo.create(restock_movement).await.unwrap();
        assert_eq!(result2.balance_after, 150); // 100 + 50

        // Sale movement
        let sale_movement = NewStockMovement {
            order_id: Some(order.id),
            product_id: product.id,
            r#type: StockMovementType::Sale,
            quantity: -30,
            created_by: admin_user.id,
            description: Some("Sale".to_string()),
            reference_id: None,
        };
        let result3 = repo.create(sale_movement).await.unwrap();
        assert_eq!(result3.balance_after, 120); // 150 - 30

        // Return movement
        let return_movement = NewStockMovement {
            order_id: Some(order.id),
            product_id: product.id,
            r#type: StockMovementType::Return,
            quantity: 10,
            created_by: admin_user.id,
            description: Some("Customer return".to_string()),
            reference_id: None,
        };
        let result4 = repo.create(return_movement).await.unwrap();
        assert_eq!(result4.balance_after, 130); // 120 + 10

        // Adjustment movement
        let adjustment_movement = NewStockMovement {
            order_id: None,
            product_id: product.id,
            r#type: StockMovementType::Adjustment,
            quantity: -5,
            created_by: admin_user.id,
            description: Some("Adjustment".to_string()),
            reference_id: None,
        };
        let result5 = repo.create(adjustment_movement).await.unwrap();
        assert_eq!(result5.balance_after, 125); // 130 - 5
    }

    #[sqlx::test]
    async fn test_chk_quantity_sign_constraints(pool: PgPool) {
        let admin_user = create_test_user(&pool, "stock_admin_chk_sign").await;
        let product = create_test_product(&pool, "test_product_chk_sign", admin_user.id).await;

        let repo = StockMovementRepository::new(Arc::new(pool.clone()));

        // Test initial movement with negative quantity (should fail)
        let invalid_initial = NewStockMovement {
            order_id: None,
            product_id: product.id,
            r#type: StockMovementType::Initial,
            quantity: -10,
            created_by: admin_user.id,
            description: None,
            reference_id: None,
        };
        let result = repo.create(invalid_initial).await;
        assert!(result.is_err()); // Expect a database error

        // Test restock movement with negative quantity (should fail)
        let invalid_restock = NewStockMovement {
            order_id: None,
            product_id: product.id,
            r#type: StockMovementType::Restock,
            quantity: -5,
            created_by: admin_user.id,
            description: None,
            reference_id: None,
        };
        let result = repo.create(invalid_restock).await;
        assert!(result.is_err()); // Expect a database error

        // Test sale movement with zero quantity (should fail based on `quantity != 0`)
        let bot = create_test_bot(&pool).await;
        let customer = create_test_customer(&pool, 22222, bot.id).await;
        let order = create_test_order(&pool, customer.id, bot.id).await;
        let invalid_sale = NewStockMovement {
            order_id: Some(order.id),
            product_id: product.id,
            r#type: StockMovementType::Sale,
            quantity: 0,
            created_by: admin_user.id,
            description: None,
            reference_id: None,
        };
        let result = repo.create(invalid_sale).await;
        assert!(result.is_err()); // Expect a database error

        // Test valid initial movement
        let valid_initial = NewStockMovement {
            order_id: None,
            product_id: product.id,
            r#type: StockMovementType::Initial,
            quantity: 10,
            created_by: admin_user.id,
            description: None,
            reference_id: None,
        };
        assert!(repo.create(valid_initial).await.is_ok());
    }

    #[sqlx::test]
    async fn test_chk_order_id_constraints(pool: PgPool) {
        let admin_user = create_test_user(&pool, "stock_admin_chk_order").await;
        let product = create_test_product(&pool, "test_product_chk_order", admin_user.id).await;

        let repo = StockMovementRepository::new(Arc::new(pool.clone()));
        let initial = NewStockMovement {
            order_id: None,
            product_id: product.id,
            r#type: StockMovementType::Initial,
            quantity: 10,
            created_by: admin_user.id,
            description: None,
            reference_id: None,
        };
        repo.create(initial).await.unwrap();

        // Test sale movement without order_id (should fail)
        let invalid_sale = NewStockMovement {
            order_id: None,
            product_id: product.id,
            r#type: StockMovementType::Sale,
            quantity: -1,
            created_by: admin_user.id,
            description: None,
            reference_id: None,
        };
        let result = repo.create(invalid_sale).await;
        assert!(result.is_err()); // Expect a database error

        // Test return movement without order_id (should fail)
        let invalid_return = NewStockMovement {
            order_id: None,
            product_id: product.id,
            r#type: StockMovementType::Return,
            quantity: 1,
            created_by: admin_user.id,
            description: None,
            reference_id: None,
        };
        let result = repo.create(invalid_return).await;
        assert!(result.is_err()); // Expect a database error

        // Test valid sale movement with order_id
        let bot = create_test_bot(&pool).await;
        let customer = create_test_customer(&pool, 33333, bot.id).await;
        let order = create_test_order(&pool, customer.id, bot.id).await;
        let valid_sale = NewStockMovement {
            order_id: Some(order.id),
            product_id: product.id,
            r#type: StockMovementType::Sale,
            quantity: -1,
            created_by: admin_user.id,
            description: None,
            reference_id: None,
        };
        assert!(repo.create(valid_sale).await.is_ok());
    }

    #[sqlx::test]
    async fn test_get_last_by_product_id(pool: PgPool) {
        let admin_user = create_test_user(&pool, "stock_admin_get_last").await;
        let product1 = create_test_product(&pool, "product_get_last1", admin_user.id).await;
        let product2 = create_test_product(&pool, "product_get_last2", admin_user.id).await;

        let repo = StockMovementRepository::new(Arc::new(pool.clone()));

        // Movement for product1
        let movement1_p1 = NewStockMovement {
            order_id: None,
            product_id: product1.id,
            r#type: StockMovementType::Initial,
            quantity: 10,
            created_by: admin_user.id,
            description: Some("Initial P1".to_string()),
            reference_id: None,
        };
        repo.create(movement1_p1).await.unwrap();

        let movement2_p1 = NewStockMovement {
            order_id: None,
            product_id: product1.id,
            r#type: StockMovementType::Adjustment,
            quantity: 5,
            created_by: admin_user.id,
            description: Some("Adjustment P1".to_string()),
            reference_id: None,
        };
        let last_p1 = repo.create(movement2_p1).await.unwrap();

        // Movement for product2
        let movement1_p2 = NewStockMovement {
            order_id: None,
            product_id: product2.id,
            r#type: StockMovementType::Initial,
            quantity: 20,
            created_by: admin_user.id,
            description: Some("Initial P2".to_string()),
            reference_id: None,
        };
        let last_p2 = repo.create(movement1_p2).await.unwrap();

        // Get last for product1
        let fetched_last_p1 = repo.get_last_by_product_id(product1.id).await.unwrap();
        assert_eq!(fetched_last_p1.id, last_p1.id);
        assert_eq!(fetched_last_p1.balance_after, 15); // 10 + 5

        // Get last for product2
        let fetched_last_p2 = repo.get_last_by_product_id(product2.id).await.unwrap();
        assert_eq!(fetched_last_p2.id, last_p2.id);
        assert_eq!(fetched_last_p2.balance_after, 20);
    }
}
