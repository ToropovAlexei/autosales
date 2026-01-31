use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::{
    errors::repository::RepositoryResult,
    models::dashboard::{CategorySalesRow, TimeSeriesRow, TopProductRow},
};

#[async_trait]
pub trait DashboardRepositoryTrait {
    async fn count_total_users(&self) -> RepositoryResult<i64>;
    async fn count_users_with_purchases(&self) -> RepositoryResult<i64>;
    async fn count_available_products(&self) -> RepositoryResult<i64>;
    async fn count_total_users_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<i64>;
    async fn count_users_with_purchases_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<i64>;
    async fn count_products_sold_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<i64>;
    async fn get_sales_count_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<i64>;
    async fn get_total_revenue_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Decimal>;
    async fn get_sales_over_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Vec<TimeSeriesRow>>;
    async fn get_users_over_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Vec<TimeSeriesRow>>;
    async fn get_revenue_over_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Vec<TimeSeriesRow>>;
    async fn get_users_with_purchases_over_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Vec<TimeSeriesRow>>;
    async fn get_top_products(&self, limit: i64) -> RepositoryResult<Vec<TopProductRow>>;
    async fn get_sales_by_category(&self) -> RepositoryResult<Vec<CategorySalesRow>>;
}

#[derive(Clone)]
pub struct DashboardRepository {
    pool: Arc<PgPool>,
}

impl DashboardRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DashboardRepositoryTrait for DashboardRepository {
    async fn count_total_users(&self) -> RepositoryResult<i64> {
        let total = sqlx::query_scalar!("SELECT COUNT(*) FROM customers")
            .fetch_one(&*self.pool)
            .await?;
        Ok(total.unwrap_or_default())
    }

    async fn count_users_with_purchases(&self) -> RepositoryResult<i64> {
        let total = sqlx::query_scalar!("SELECT COUNT(DISTINCT customer_id) FROM orders")
            .fetch_one(&*self.pool)
            .await?;
        Ok(total.unwrap_or_default())
    }

    async fn count_available_products(&self) -> RepositoryResult<i64> {
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM products WHERE stock > 0 AND deleted_at IS NULL"
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(total.unwrap_or_default())
    }

    async fn count_total_users_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<i64> {
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM customers WHERE created_at >= $1 AND created_at <= $2",
            start,
            end
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(total.unwrap_or_default())
    }

    async fn count_users_with_purchases_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<i64> {
        let total = sqlx::query_scalar!(
            "SELECT COUNT(DISTINCT customer_id) FROM orders WHERE created_at >= $1 AND created_at <= $2",
            start,
            end
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(total.unwrap_or_default())
    }

    async fn count_products_sold_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<i64> {
        let total = sqlx::query_scalar!(
            r#"
            SELECT COALESCE(SUM(oi.quantity)::BIGINT, 0)
            FROM order_items oi
            JOIN orders o ON o.id = oi.order_id
            WHERE o.created_at >= $1 AND o.created_at <= $2
            "#,
            start,
            end
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(total.unwrap_or_default())
    }

    async fn get_sales_count_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<i64> {
        self.count_products_sold_for_period(start, end).await
    }

    async fn get_total_revenue_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Decimal> {
        let total = sqlx::query_scalar!(
            r#"
            SELECT COALESCE(SUM(oi.price_at_purchase * oi.quantity), 0)
            FROM order_items oi
            JOIN orders o ON o.id = oi.order_id
            WHERE o.created_at >= $1 AND o.created_at <= $2
            "#,
            start,
            end
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(total.unwrap_or_default())
    }

    async fn get_sales_over_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Vec<TimeSeriesRow>> {
        let rows = sqlx::query_as!(
            TimeSeriesRow,
            r#"
            SELECT DATE(o.created_at) as "date!", COALESCE(SUM(oi.quantity)::BIGINT, 0) as "value!"
            FROM order_items oi
            JOIN orders o ON o.id = oi.order_id
            WHERE o.created_at >= $1 AND o.created_at <= $2
            GROUP BY DATE(o.created_at)
            ORDER BY DATE(o.created_at)
            "#,
            start,
            end
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    async fn get_users_over_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Vec<TimeSeriesRow>> {
        let rows = sqlx::query_as!(
            TimeSeriesRow,
            r#"
            SELECT DATE(created_at) as "date!", COUNT(*)::BIGINT as "value!"
            FROM customers
            WHERE created_at >= $1 AND created_at <= $2
            GROUP BY DATE(created_at)
            ORDER BY DATE(created_at)
            "#,
            start,
            end
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    async fn get_revenue_over_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Vec<TimeSeriesRow>> {
        let rows = sqlx::query_as!(
            TimeSeriesRow,
            r#"
            SELECT DATE(o.created_at) as "date!", COALESCE(SUM(oi.price_at_purchase * oi.quantity), 0)::BIGINT as "value!"
            FROM order_items oi
            JOIN orders o ON o.id = oi.order_id
            WHERE o.created_at >= $1 AND o.created_at <= $2
            GROUP BY DATE(o.created_at)
            ORDER BY DATE(o.created_at)
            "#,
            start,
            end
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    async fn get_users_with_purchases_over_time(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RepositoryResult<Vec<TimeSeriesRow>> {
        let rows = sqlx::query_as!(
            TimeSeriesRow,
            r#"
            SELECT DATE(created_at) as "date!", COUNT(DISTINCT customer_id)::BIGINT as "value!"
            FROM orders
            WHERE created_at >= $1 AND created_at <= $2
            GROUP BY DATE(created_at)
            ORDER BY DATE(created_at)
            "#,
            start,
            end
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    async fn get_top_products(&self, limit: i64) -> RepositoryResult<Vec<TopProductRow>> {
        let rows = sqlx::query_as!(
            TopProductRow,
            r#"
            SELECT
                p.id,
                p.name,
                COALESCE(AVG(oi.price_at_purchase), 0) as "price!",
                COALESCE(SUM(oi.price_at_purchase * oi.quantity), 0) as "total_revenue!"
            FROM order_items oi
            JOIN orders o ON o.id = oi.order_id
            JOIN products p ON p.id = oi.product_id
            GROUP BY p.id, p.name
            ORDER BY COALESCE(SUM(oi.price_at_purchase * oi.quantity), 0) DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    async fn get_sales_by_category(&self) -> RepositoryResult<Vec<CategorySalesRow>> {
        let rows = sqlx::query_as!(
            CategorySalesRow,
            r#"
            SELECT
                c.name as "category_name?",
                COALESCE(SUM(oi.price_at_purchase * oi.quantity), 0) as "total_sales!"
            FROM order_items oi
            JOIN orders o ON o.id = oi.order_id
            JOIN products p ON p.id = oi.product_id
            LEFT JOIN categories c ON c.id = p.category_id
            GROUP BY c.name
            ORDER BY COALESCE(SUM(oi.price_at_purchase * oi.quantity), 0) DESC
            "#
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use chrono::{Duration, Utc};
    use rust_decimal::Decimal;
    use sqlx::PgPool;

    async fn create_customer(pool: &PgPool, telegram_id: i64, created_at: DateTime<Utc>) -> i64 {
        let id = sqlx::query_scalar!(
            "INSERT INTO customers (telegram_id, registered_with_bot, last_seen_with_bot) VALUES ($1, 1, 1) RETURNING id",
            telegram_id
        )
        .fetch_one(pool)
        .await
        .unwrap();

        sqlx::query!(
            "UPDATE customers SET created_at = $2 WHERE id = $1",
            id,
            created_at
        )
        .execute(pool)
        .await
        .unwrap();

        id
    }

    async fn create_bot(pool: &PgPool, token: &str, username: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO bots (owner_id, token, username, type, is_active, is_primary, referral_percentage)
            VALUES (NULL, $1, $2, 'main', true, false, 0.0)
            RETURNING id
            "#,
            token,
            username
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_category(pool: &PgPool, name: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO categories (name, parent_id, image_id, created_by)
            VALUES ($1, NULL, NULL, 1)
            RETURNING id
            "#,
            name
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_product(
        pool: &PgPool,
        name: &str,
        category_id: Option<i64>,
        stock: i32,
    ) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO products (
                name, base_price, category_id, image_id, stock, type,
                subscription_period_days, details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by
            )
            VALUES ($1, 10.0, $2, NULL, $3, 'item', 0, NULL, NULL, NULL, 'internal', NULL, 1)
            RETURNING id
            "#,
            name,
            category_id,
            stock
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_order(
        pool: &PgPool,
        customer_id: i64,
        bot_id: i64,
        created_at: DateTime<Utc>,
    ) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO orders (
                customer_id, amount, currency, status, bot_id, created_at, updated_at
            )
            VALUES ($1, 100.0, 'RUB', 'fulfilled', $2, $3, $3)
            RETURNING id
            "#,
            customer_id,
            bot_id,
            created_at
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_order_item(
        pool: &PgPool,
        order_id: i64,
        product_id: i64,
        price: &str,
        qty: i16,
    ) {
        sqlx::query!(
            r#"
            INSERT INTO order_items (
                order_id, product_id, name_at_purchase, price_at_purchase, quantity,
                fulfillment_type, fulfillment_content, fulfillment_image_id, details
            )
            VALUES ($1, $2, $3, $4, $5, 'text', NULL, NULL, NULL)
            "#,
            order_id,
            product_id,
            "Test Product",
            Decimal::from_str(price).unwrap(),
            qty
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[sqlx::test]
    async fn test_basic_counts(pool: PgPool) {
        let repo = DashboardRepository::new(Arc::new(pool.clone()));

        let now = Utc::now();
        let c1 = create_customer(&pool, 10001, now).await;
        let _c2 = create_customer(&pool, 10002, now).await;
        let bot_id = create_bot(&pool, "dash_bot", "dash_bot").await;

        let cat_id = create_category(&pool, "Games").await;
        let p1 = create_product(&pool, "P1", Some(cat_id), 5).await;
        let _p2 = create_product(&pool, "P2", None, 0).await;

        let order_id = create_order(&pool, c1, bot_id, now).await;
        create_order_item(&pool, order_id, p1, "15.00", 2).await;

        assert_eq!(repo.count_total_users().await.unwrap(), 2);
        assert_eq!(repo.count_users_with_purchases().await.unwrap(), 1);
        assert_eq!(repo.count_available_products().await.unwrap(), 1);
        assert_eq!(
            repo.count_products_sold_for_period(now - Duration::days(1), now + Duration::days(1))
                .await
                .unwrap(),
            2
        );
    }

    #[sqlx::test]
    async fn test_time_series_and_top(pool: PgPool) {
        let repo = DashboardRepository::new(Arc::new(pool.clone()));
        let now = Utc::now();
        let day1 = now - Duration::days(2);
        let day2 = now - Duration::days(1);
        let day3 = now;

        let customer_id = create_customer(&pool, 20001, day1).await;
        let bot_id = create_bot(&pool, "dash_bot2", "dash_bot2").await;
        let cat_id = create_category(&pool, "Cards").await;
        let p1 = create_product(&pool, "TopProduct", Some(cat_id), 5).await;
        let p2 = create_product(&pool, "SecondProduct", None, 5).await;

        let order1 = create_order(&pool, customer_id, bot_id, day2).await;
        let order2 = create_order(&pool, customer_id, bot_id, day3).await;
        create_order_item(&pool, order1, p1, "10.00", 1).await;
        create_order_item(&pool, order2, p1, "10.00", 2).await;
        create_order_item(&pool, order2, p2, "5.00", 3).await;

        let revenue = repo
            .get_total_revenue_for_period(day1, day3 + Duration::days(1))
            .await
            .unwrap();
        assert_eq!(revenue, Decimal::from_str("45.00").unwrap());

        let sales = repo
            .get_sales_over_time(day1, day3 + Duration::days(1))
            .await
            .unwrap();
        assert!(!sales.is_empty());

        let top = repo.get_top_products(2).await.unwrap();
        assert_eq!(top[0].id, p1);

        let categories = repo.get_sales_by_category().await.unwrap();
        assert!(
            categories
                .iter()
                .any(|c| c.category_name.as_deref() == Some("Cards"))
        );
    }
}
