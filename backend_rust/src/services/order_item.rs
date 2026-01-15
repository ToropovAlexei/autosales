use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::ApiResult,
    infrastructure::repositories::{
        order_item::{OrderItemRepository, OrderItemRepositoryTrait},
        stock_movement::{StockMovementRepository, StockMovementRepositoryTrait},
    },
    models::{
        order_item::{NewOrderItem, OrderItemRow},
        stock_movement::{NewStockMovement, StockMovementType},
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
