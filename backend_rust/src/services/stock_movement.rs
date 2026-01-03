use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::stock_movement::{
        StockMovementRepository, StockMovementRepositoryTrait,
    },
    models::{
        common::PaginatedResult,
        stock_movement::{StockMovementListQuery, StockMovementRow},
    },
};

#[async_trait]
pub trait StockMovementServiceTrait: Send + Sync {
    async fn get_list(
        &self,
        query: StockMovementListQuery,
    ) -> ApiResult<PaginatedResult<StockMovementRow>>;
}

pub struct StockMovementService<S> {
    stock_movement_repo: Arc<S>,
}

impl<S> StockMovementService<S>
where
    S: StockMovementRepositoryTrait + Send + Sync,
{
    pub fn new(stock_movement_repo: Arc<S>) -> Self {
        Self {
            stock_movement_repo,
        }
    }
}

#[async_trait]
impl StockMovementServiceTrait for StockMovementService<StockMovementRepository> {
    async fn get_list(
        &self,
        query: StockMovementListQuery,
    ) -> ApiResult<PaginatedResult<StockMovementRow>> {
        self.stock_movement_repo
            .get_list(query)
            .await
            .map_err(ApiError::from)
    }
}
