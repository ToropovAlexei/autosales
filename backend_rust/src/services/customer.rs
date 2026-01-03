use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::customer::{CustomerRepository, CustomerRepositoryTrait},
    models::{
        common::PaginatedResult,
        customer::{CustomerListQuery, CustomerRow, NewCustomer, UpdateCustomer},
    },
};

#[async_trait]
pub trait CustomerServiceTrait: Send + Sync {
    async fn get_list(&self, query: CustomerListQuery) -> ApiResult<PaginatedResult<CustomerRow>>;
    async fn create(&self, customer: NewCustomer) -> ApiResult<CustomerRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<CustomerRow>;
    async fn get_by_telegram_id(&self, id: i64) -> ApiResult<CustomerRow>;
    async fn update(&self, id: i64, customer: UpdateCustomer) -> ApiResult<CustomerRow>;
}

pub struct CustomerService<R> {
    customer_repo: Arc<R>,
}

impl<R> CustomerService<R>
where
    R: CustomerRepositoryTrait + Send + Sync,
{
    pub fn new(customer_repo: Arc<R>) -> Self {
        Self { customer_repo }
    }
}

#[async_trait]
impl CustomerServiceTrait for CustomerService<CustomerRepository> {
    async fn get_list(&self, query: CustomerListQuery) -> ApiResult<PaginatedResult<CustomerRow>> {
        self.customer_repo
            .get_list(query)
            .await
            .map_err(ApiError::from)
    }

    async fn create(&self, customer: NewCustomer) -> ApiResult<CustomerRow> {
        let created = self.customer_repo.create(customer).await?;

        Ok(created)
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<CustomerRow> {
        let res = self.customer_repo.get_by_id(id).await?;
        Ok(res)
    }

    async fn get_by_telegram_id(&self, id: i64) -> ApiResult<CustomerRow> {
        let res = self.customer_repo.get_by_telegram_id(id).await?;
        Ok(res)
    }

    async fn update(&self, id: i64, customer: UpdateCustomer) -> ApiResult<CustomerRow> {
        let updated = self.customer_repo.update(id, customer).await?;

        Ok(updated)
    }
}
