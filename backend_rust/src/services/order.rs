use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        order::{OrderRepository, OrderRepositoryTrait},
        order_item::{OrderItemRepository, OrderItemRepositoryTrait},
    },
    models::{
        common::PaginatedResult,
        order::{NewOrder, OrderListQuery, OrderRow, OrderStatus},
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
    async fn get_list(&self, query: OrderListQuery) -> ApiResult<PaginatedResult<OrderRow>>;
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
    async fn get_list(&self, query: OrderListQuery) -> ApiResult<PaginatedResult<OrderRow>> {
        self.order_repo
            .get_list(query)
            .await
            .map_err(ApiError::from)
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
