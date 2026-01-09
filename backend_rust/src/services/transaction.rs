use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::transaction::{
        TransactionRepository, TransactionRepositoryTrait,
    },
    models::{
        common::PaginatedResult,
        transaction::{NewTransaction, TransactionListQuery, TransactionRow},
    },
};

#[async_trait]
pub trait TransactionServiceTrait: Send + Sync {
    async fn get_list(
        &self,
        query: TransactionListQuery,
    ) -> ApiResult<PaginatedResult<TransactionRow>>;
    async fn create(&self, transaction: NewTransaction) -> ApiResult<TransactionRow>;
    async fn get_last(&self) -> ApiResult<TransactionRow>;
}

pub struct TransactionService<R> {
    repo: Arc<R>,
}

impl<R> TransactionService<R>
where
    R: TransactionRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl TransactionServiceTrait for TransactionService<TransactionRepository> {
    async fn get_list(
        &self,
        query: TransactionListQuery,
    ) -> ApiResult<PaginatedResult<TransactionRow>> {
        self.repo.get_list(query).await.map_err(ApiError::from)
    }

    async fn create(&self, transaction: NewTransaction) -> ApiResult<TransactionRow> {
        let created = self.repo.create(transaction).await?;
        Ok(created)
    }

    async fn get_last(&self) -> ApiResult<TransactionRow> {
        self.repo.get_last().await.map_err(ApiError::from)
    }
}
