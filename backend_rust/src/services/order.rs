use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::order::{OrderRepository, OrderRepositoryTrait},
    models::{
        common::PaginatedResult,
        order::{OrderListQuery, OrderRow},
    },
};

#[async_trait]
pub trait OrderServiceTrait: Send + Sync {
    async fn get_list(&self, query: OrderListQuery) -> ApiResult<PaginatedResult<OrderRow>>;
    async fn get_by_id(&self, id: i64) -> ApiResult<OrderRow>;
}

pub struct OrderService<R> {
    order_repo: Arc<R>,
}

impl<R> OrderService<R>
where
    R: OrderRepositoryTrait + Send + Sync,
{
    pub fn new(order_repo: Arc<R>) -> Self {
        Self { order_repo }
    }
}

#[async_trait]
impl OrderServiceTrait for OrderService<OrderRepository> {
    async fn get_list(&self, query: OrderListQuery) -> ApiResult<PaginatedResult<OrderRow>> {
        self.order_repo
            .get_list(query)
            .await
            .map_err(ApiError::from)
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<OrderRow> {
        let res = self.order_repo.get_by_id(id).await?;
        Ok(res)
    }
}
