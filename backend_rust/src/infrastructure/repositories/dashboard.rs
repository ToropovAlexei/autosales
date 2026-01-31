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
