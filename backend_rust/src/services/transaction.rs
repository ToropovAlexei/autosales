use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::transaction::{
        TransactionRepository, TransactionRepositoryTrait,
    },
    models::{
        common::{ListQuery, PaginatedResult},
        transaction::{NewTransaction, TransactionRow},
    },
};

#[async_trait]
pub trait TransactionServiceTrait: Send + Sync {
    async fn get_list(&self, query: ListQuery) -> ApiResult<PaginatedResult<TransactionRow>>;
    async fn create(&self, transaction: NewTransaction) -> ApiResult<TransactionRow>;
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
    async fn get_list(&self, query: ListQuery) -> ApiResult<PaginatedResult<TransactionRow>> {
        self.repo.get_list(query).await.map_err(ApiError::from)
    }

    async fn create(&self, transaction: NewTransaction) -> ApiResult<TransactionRow> {
        let created = self.repo.create(transaction).await?;
        Ok(created)
    }
}
