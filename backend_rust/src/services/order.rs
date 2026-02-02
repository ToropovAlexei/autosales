use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use shared_dtos::order::OrderStatus;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        order::{OrderRepository, OrderRepositoryTrait},
        order_item::{OrderItemRepository, OrderItemRepositoryTrait},
    },
    models::{
        common::PaginatedResult,
        order::{NewOrder, OrderListQuery, OrderRow},
        order_item::OrderItemRow,
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct EnrichedOrder {
    pub id: i64,
    pub customer_id: i64,
    pub amount: Decimal,
    pub currency: String,
    pub status: OrderStatus,
    pub bot_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub order_items: Vec<OrderItemRow>,
}

#[async_trait]
pub trait OrderServiceTrait: Send + Sync {
    async fn get_list(&self, query: OrderListQuery) -> ApiResult<PaginatedResult<EnrichedOrder>>;
    async fn get_for_customer(&self, customer_id: i64) -> ApiResult<Vec<EnrichedOrder>>;
    async fn get_by_id(&self, id: i64) -> ApiResult<EnrichedOrder>;
    async fn create(&self, order: NewOrder) -> ApiResult<OrderRow>;
}

pub struct OrderService<R, OI> {
    order_repo: Arc<R>,
    order_item_repo: Arc<OI>,
}

impl<R, OI> OrderService<R, OI>
where
    R: OrderRepositoryTrait + Send + Sync,
    OI: OrderItemRepositoryTrait + Send + Sync,
{
    pub fn new(order_repo: Arc<R>, order_item_repo: Arc<OI>) -> Self {
        Self {
            order_repo,
            order_item_repo,
        }
    }
}

#[async_trait]
impl OrderServiceTrait for OrderService<OrderRepository, OrderItemRepository> {
    async fn get_list(&self, query: OrderListQuery) -> ApiResult<PaginatedResult<EnrichedOrder>> {
        let orders = self
            .order_repo
            .get_list(query)
            .await
            .map_err(ApiError::from)?;

        let order_items = self
            .order_item_repo
            .get_for_orders(orders.items.iter().map(|o| o.id).collect())
            .await
            .map_err(ApiError::from)?;
        let order_items_by_order_id: HashMap<i64, Vec<OrderItemRow>> =
            order_items
                .iter()
                .fold(HashMap::new(), |mut acc, order_item| {
                    acc.entry(order_item.order_id)
                        .or_insert_with(Vec::new)
                        .push(order_item.clone());
                    acc
                });

        Ok(PaginatedResult {
            total: orders.items.len() as i64,
            items: orders
                .items
                .iter()
                .map(|order| EnrichedOrder {
                    id: order.id,
                    customer_id: order.customer_id,
                    amount: order.amount,
                    currency: order.currency.clone(),
                    status: order.status,
                    bot_id: order.bot_id,
                    created_at: order.created_at,
                    updated_at: order.updated_at,
                    paid_at: order.paid_at,
                    fulfilled_at: order.fulfilled_at,
                    cancelled_at: order.cancelled_at,
                    order_items: order_items_by_order_id
                        .get(&order.id)
                        .cloned()
                        .unwrap_or_default(),
                })
                .collect::<Vec<EnrichedOrder>>(),
        })
    }

    async fn get_for_customer(&self, customer_id: i64) -> ApiResult<Vec<EnrichedOrder>> {
        let orders = self
            .order_repo
            .get_for_customer(customer_id)
            .await
            .map_err(ApiError::from)?;
        let order_items = self
            .order_item_repo
            .get_for_orders(orders.iter().map(|o| o.id).collect())
            .await
            .map_err(ApiError::from)?;
        let order_items_by_order_id: HashMap<i64, Vec<OrderItemRow>> =
            order_items
                .iter()
                .fold(HashMap::new(), |mut acc, order_item| {
                    acc.entry(order_item.order_id)
                        .or_insert_with(Vec::new)
                        .push(order_item.clone());
                    acc
                });

        Ok(orders
            .iter()
            .map(|order| EnrichedOrder {
                id: order.id,
                customer_id: order.customer_id,
                amount: order.amount,
                currency: order.currency.clone(),
                status: order.status,
                bot_id: order.bot_id,
                created_at: order.created_at,
                updated_at: order.updated_at,
                paid_at: order.paid_at,
                fulfilled_at: order.fulfilled_at,
                cancelled_at: order.cancelled_at,
                order_items: order_items_by_order_id
                    .get(&order.id)
                    .cloned()
                    .unwrap_or_default(),
            })
            .collect::<Vec<EnrichedOrder>>())
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<EnrichedOrder> {
        let order = self.order_repo.get_by_id(id).await?;
        let order_items = self
            .order_item_repo
            .get_for_order(order.id)
            .await
            .map_err(ApiError::from)?;
        Ok(EnrichedOrder {
            id: order.id,
            customer_id: order.customer_id,
            amount: order.amount,
            currency: order.currency.clone(),
            status: order.status,
            bot_id: order.bot_id,
            created_at: order.created_at,
            updated_at: order.updated_at,
            paid_at: order.paid_at,
            fulfilled_at: order.fulfilled_at,
            cancelled_at: order.cancelled_at,
            order_items,
        })
    }

    async fn create(&self, order: NewOrder) -> ApiResult<OrderRow> {
        let res = self.order_repo.create(order).await?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::{
        order::OrderRepository, order_item::OrderItemRepository,
    };
    use crate::models::order_item::NewOrderItem;
    use rust_decimal::Decimal;
    use sqlx::PgPool;
    use std::collections::HashMap;
    use std::sync::Arc;

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

    fn build_service(pool: &PgPool) -> OrderService<OrderRepository, OrderItemRepository> {
        let pool = Arc::new(pool.clone());
        OrderService::new(
            Arc::new(OrderRepository::new(pool.clone())),
            Arc::new(OrderItemRepository::new(pool.clone())),
        )
    }

    #[sqlx::test]
    async fn test_get_for_customer_enriches_items(pool: PgPool) {
        let service = build_service(&pool);
        let order_repo = OrderRepository::new(Arc::new(pool.clone()));
        let order_item_repo = OrderItemRepository::new(Arc::new(pool.clone()));

        let customer_id = create_customer(&pool, 10101).await;
        let other_customer_id = create_customer(&pool, 20202).await;
        let bot_id = create_bot(&pool, Some(customer_id), "order_svc_bot", "order_svc_bot").await;
        let product_id = create_product(&pool, "order_svc_product").await;

        let order1 = order_repo
            .create(NewOrder {
                customer_id,
                amount: Decimal::from(100),
                currency: "USD".to_string(),
                status: OrderStatus::Created,
                bot_id,
                paid_at: None,
                fulfilled_at: None,
            })
            .await
            .unwrap();

        let order2 = order_repo
            .create(NewOrder {
                customer_id,
                amount: Decimal::from(150),
                currency: "USD".to_string(),
                status: OrderStatus::Created,
                bot_id,
                paid_at: None,
                fulfilled_at: None,
            })
            .await
            .unwrap();

        let other_order = order_repo
            .create(NewOrder {
                customer_id: other_customer_id,
                amount: Decimal::from(200),
                currency: "USD".to_string(),
                status: OrderStatus::Created,
                bot_id,
                paid_at: None,
                fulfilled_at: None,
            })
            .await
            .unwrap();

        order_item_repo
            .create(NewOrderItem {
                order_id: order1.id,
                product_id,
                name_at_purchase: "Item A".to_string(),
                price_at_purchase: Decimal::from(10),
                quantity: 1,
                fulfillment_type: "text".to_string(),
                fulfillment_content: None,
                fulfillment_image_id: None,
                details: None,
            })
            .await
            .unwrap();

        order_item_repo
            .create(NewOrderItem {
                order_id: order2.id,
                product_id,
                name_at_purchase: "Item B".to_string(),
                price_at_purchase: Decimal::from(20),
                quantity: 1,
                fulfillment_type: "text".to_string(),
                fulfillment_content: None,
                fulfillment_image_id: None,
                details: None,
            })
            .await
            .unwrap();

        order_item_repo
            .create(NewOrderItem {
                order_id: order2.id,
                product_id,
                name_at_purchase: "Item C".to_string(),
                price_at_purchase: Decimal::from(30),
                quantity: 2,
                fulfillment_type: "text".to_string(),
                fulfillment_content: None,
                fulfillment_image_id: None,
                details: None,
            })
            .await
            .unwrap();

        order_item_repo
            .create(NewOrderItem {
                order_id: other_order.id,
                product_id,
                name_at_purchase: "Item D".to_string(),
                price_at_purchase: Decimal::from(40),
                quantity: 1,
                fulfillment_type: "text".to_string(),
                fulfillment_content: None,
                fulfillment_image_id: None,
                details: None,
            })
            .await
            .unwrap();

        let enriched = service.get_for_customer(customer_id).await.unwrap();
        assert_eq!(enriched.len(), 2);

        let items_by_order: HashMap<i64, usize> = enriched
            .iter()
            .map(|order| (order.id, order.order_items.len()))
            .collect();
        assert_eq!(items_by_order.get(&order1.id), Some(&1));
        assert_eq!(items_by_order.get(&order2.id), Some(&2));
    }

    #[sqlx::test]
    async fn test_get_by_id_includes_items(pool: PgPool) {
        let service = build_service(&pool);
        let order_repo = OrderRepository::new(Arc::new(pool.clone()));
        let order_item_repo = OrderItemRepository::new(Arc::new(pool.clone()));

        let customer_id = create_customer(&pool, 30303).await;
        let bot_id = create_bot(
            &pool,
            Some(customer_id),
            "order_svc_bot_2",
            "order_svc_bot_2",
        )
        .await;
        let product_id = create_product(&pool, "order_svc_product_2").await;

        let order = order_repo
            .create(NewOrder {
                customer_id,
                amount: Decimal::from(100),
                currency: "USD".to_string(),
                status: OrderStatus::Created,
                bot_id,
                paid_at: None,
                fulfilled_at: None,
            })
            .await
            .unwrap();

        order_item_repo
            .create(NewOrderItem {
                order_id: order.id,
                product_id,
                name_at_purchase: "Item A".to_string(),
                price_at_purchase: Decimal::from(10),
                quantity: 1,
                fulfillment_type: "text".to_string(),
                fulfillment_content: None,
                fulfillment_image_id: None,
                details: None,
            })
            .await
            .unwrap();

        order_item_repo
            .create(NewOrderItem {
                order_id: order.id,
                product_id,
                name_at_purchase: "Item B".to_string(),
                price_at_purchase: Decimal::from(20),
                quantity: 1,
                fulfillment_type: "text".to_string(),
                fulfillment_content: None,
                fulfillment_image_id: None,
                details: None,
            })
            .await
            .unwrap();

        let enriched = service.get_by_id(order.id).await.unwrap();
        assert_eq!(enriched.id, order.id);
        assert_eq!(enriched.order_items.len(), 2);
    }

    #[sqlx::test]
    async fn test_create_order(pool: PgPool) {
        let service = build_service(&pool);
        let customer_id = create_customer(&pool, 40404).await;
        let bot_id = create_bot(
            &pool,
            Some(customer_id),
            "order_svc_bot_3",
            "order_svc_bot_3",
        )
        .await;

        let created = service
            .create(NewOrder {
                customer_id,
                amount: Decimal::from(250),
                currency: "USD".to_string(),
                status: OrderStatus::Created,
                bot_id,
                paid_at: None,
                fulfilled_at: None,
            })
            .await
            .unwrap();

        assert_eq!(created.customer_id, customer_id);
        assert_eq!(created.amount, Decimal::from(250));
        assert_eq!(created.currency, "USD");
        assert_eq!(created.status, OrderStatus::Created);
        assert_eq!(created.bot_id, bot_id);
    }
}
