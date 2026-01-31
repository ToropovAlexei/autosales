use std::sync::Arc;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::dashboard::{DashboardRepository, DashboardRepositoryTrait},
    models::dashboard::{CategorySalesRow, TimeSeriesRow, TopProductRow},
    presentation::admin::dtos::dashboard::{
        CategorySalesResponse, DashboardOverviewResponse, SalesOverTimeResponse,
        StatWithTrendResponse, TimeSeriesDashboardDataResponse, TimeSeriesPointResponse,
        TopProductResponse,
    },
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;

#[async_trait]
pub trait DashboardServiceTrait: Send + Sync {
    async fn get_dashboard_stats(&self) -> ApiResult<DashboardOverviewResponse>;
    async fn get_time_series_data(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> ApiResult<TimeSeriesDashboardDataResponse>;
    async fn get_top_products(&self, limit: i64) -> ApiResult<Vec<TopProductResponse>>;
    async fn get_sales_by_category(&self) -> ApiResult<Vec<CategorySalesResponse>>;
}

pub struct DashboardService<R> {
    repo: Arc<R>,
}

impl<R> DashboardService<R>
where
    R: DashboardRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    fn calc_trend(current: i64, previous: i64) -> f64 {
        if previous == 0 {
            return 0.0;
        }
        (current as f64 - previous as f64) / previous as f64 * 100.0
    }

    fn fill_missing_dates(
        start: NaiveDate,
        end: NaiveDate,
        data: Vec<TimeSeriesRow>,
    ) -> Vec<TimeSeriesPointResponse> {
        let mut map = std::collections::HashMap::new();
        for item in data {
            map.insert(item.date, item.value);
        }

        let mut res = Vec::new();
        let mut current = start;
        while current <= end {
            let value = map.get(&current).copied().unwrap_or(0);
            res.push(TimeSeriesPointResponse {
                date: current,
                value,
            });
            current = current
                .succ_opt()
                .unwrap_or(current + chrono::Duration::days(1));
        }
        res
    }
}

#[async_trait]
impl DashboardServiceTrait for DashboardService<DashboardRepository> {
    async fn get_dashboard_stats(&self) -> ApiResult<DashboardOverviewResponse> {
        let total_users = self.repo.count_total_users().await?;
        let users_with_purchases = self.repo.count_users_with_purchases().await?;
        let available_products = self.repo.count_available_products().await?;

        let end = Utc::now();
        let start = end - Duration::days(30);
        let prev_end = start;
        let prev_start = prev_end - Duration::days(30);

        let total_users_current = self.repo.count_total_users_for_period(start, end).await?;
        let total_users_previous = self
            .repo
            .count_total_users_for_period(prev_start, prev_end)
            .await?;

        let users_with_purchases_current = self
            .repo
            .count_users_with_purchases_for_period(start, end)
            .await?;
        let users_with_purchases_previous = self
            .repo
            .count_users_with_purchases_for_period(prev_start, prev_end)
            .await?;

        let products_sold_current = self.repo.count_products_sold_for_period(start, end).await?;
        let products_sold_previous = self
            .repo
            .count_products_sold_for_period(prev_start, prev_end)
            .await?;

        Ok(DashboardOverviewResponse {
            total_users,
            users_with_purchases,
            available_products,
            total_users_30_days: StatWithTrendResponse {
                value: total_users_current,
                trend: Self::calc_trend(total_users_current, total_users_previous),
            },
            users_with_purchases_30_days: StatWithTrendResponse {
                value: users_with_purchases_current,
                trend: Self::calc_trend(
                    users_with_purchases_current,
                    users_with_purchases_previous,
                ),
            },
            products_sold_30_days: StatWithTrendResponse {
                value: products_sold_current,
                trend: Self::calc_trend(products_sold_current, products_sold_previous),
            },
        })
    }

    async fn get_time_series_data(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> ApiResult<TimeSeriesDashboardDataResponse> {
        if start > end {
            return Err(ApiError::BadRequest(
                "start_date cannot be after end_date".to_string(),
            ));
        }

        let sales_count = self.repo.get_sales_count_for_period(start, end).await?;
        let total_revenue = self.repo.get_total_revenue_for_period(start, end).await?;

        let sales_raw = self.repo.get_sales_over_time(start, end).await?;
        let users_raw = self.repo.get_users_over_time(start, end).await?;
        let revenue_raw = self.repo.get_revenue_over_time(start, end).await?;
        let users_with_purchases_raw = self
            .repo
            .get_users_with_purchases_over_time(start, end)
            .await?;

        let start_date = start.date_naive();
        let end_date = end.date_naive();

        Ok(TimeSeriesDashboardDataResponse {
            sales: SalesOverTimeResponse {
                products_sold: sales_count,
                total_revenue: total_revenue.to_f64().unwrap_or_default(),
            },
            sales_chart: Self::fill_missing_dates(start_date, end_date, sales_raw),
            users_chart: Self::fill_missing_dates(start_date, end_date, users_raw),
            revenue_chart: Self::fill_missing_dates(start_date, end_date, revenue_raw),
            users_with_purchases_chart: Self::fill_missing_dates(
                start_date,
                end_date,
                users_with_purchases_raw,
            ),
        })
    }

    async fn get_top_products(&self, limit: i64) -> ApiResult<Vec<TopProductResponse>> {
        let items = self.repo.get_top_products(limit).await?;
        Ok(items.into_iter().map(TopProductResponse::from).collect())
    }

    async fn get_sales_by_category(&self) -> ApiResult<Vec<CategorySalesResponse>> {
        let items = self.repo.get_sales_by_category().await?;
        Ok(items.into_iter().map(CategorySalesResponse::from).collect())
    }
}

impl From<TopProductRow> for TopProductResponse {
    fn from(row: TopProductRow) -> Self {
        TopProductResponse {
            id: row.id,
            name: row.name,
            price: row.price.to_f64().unwrap_or_default(),
            total_revenue: row.total_revenue.to_f64().unwrap_or_default(),
        }
    }
}

impl From<CategorySalesRow> for CategorySalesResponse {
    fn from(row: CategorySalesRow) -> Self {
        CategorySalesResponse {
            category_name: row
                .category_name
                .unwrap_or_else(|| "Без категории".to_string()),
            total_sales: row.total_sales.to_f64().unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::dashboard::DashboardRepository;
    use chrono::Utc;
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

    async fn create_product(pool: &PgPool, name: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO products (
                name, base_price, category_id, image_id, stock, type,
                subscription_period_days, details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by
            )
            VALUES ($1, 10.0, NULL, NULL, 5, 'item', 0, NULL, NULL, NULL, 'internal', NULL, 1)
            RETURNING id
            "#,
            name
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

    async fn create_order_item(pool: &PgPool, order_id: i64, product_id: i64, qty: i16) {
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
            Decimal::from(10),
            qty
        )
        .execute(pool)
        .await
        .unwrap();
    }

    fn build_service(pool: &PgPool) -> DashboardService<DashboardRepository> {
        let pool = Arc::new(pool.clone());
        DashboardService::new(Arc::new(DashboardRepository::new(pool)))
    }

    #[sqlx::test]
    async fn test_time_series_fills_missing_dates(pool: PgPool) {
        let service = build_service(&pool);
        let now = Utc::now();
        let start = now - Duration::days(2);
        let end = now;

        let customer_id = create_customer(&pool, 30001, start).await;
        let bot_id = create_bot(&pool, "dash_svc_bot", "dash_svc_bot").await;
        let product_id = create_product(&pool, "dash_svc_product").await;

        let order_id = create_order(&pool, customer_id, bot_id, start + Duration::days(1)).await;
        create_order_item(&pool, order_id, product_id, 3).await;

        let data = service.get_time_series_data(start, end).await.unwrap();
        assert_eq!(data.sales.products_sold, 3);
        assert_eq!(data.sales_chart.len(), 3);
        assert_eq!(data.users_chart.len(), 3);
        assert_eq!(data.revenue_chart.len(), 3);
        assert_eq!(data.users_with_purchases_chart.len(), 3);
    }
}
