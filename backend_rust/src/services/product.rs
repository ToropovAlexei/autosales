use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::products::{ProductRepository, ProductRepositoryTrait},
    models::{
        common::PaginatedResult,
        product::{NewProduct, ProductListQuery, ProductRow, UpdateProduct},
    },
};

#[async_trait]
pub trait ProductServiceTrait: Send + Sync {
    async fn get_list(&self, query: ProductListQuery) -> ApiResult<PaginatedResult<ProductRow>>;
    async fn create(&self, product: NewProduct) -> ApiResult<ProductRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<ProductRow>;
    async fn update(&self, id: i64, product: UpdateProduct) -> ApiResult<ProductRow>;
    async fn delete(&self, id: i64) -> ApiResult<()>;
}

pub struct ProductService<R> {
    repo: Arc<R>,
}

impl<R> ProductService<R>
where
    R: ProductRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ProductServiceTrait for ProductService<ProductRepository> {
    async fn get_list(&self, query: ProductListQuery) -> ApiResult<PaginatedResult<ProductRow>> {
        self.repo.get_list(query).await.map_err(ApiError::from)
    }

    async fn create(&self, category: NewProduct) -> ApiResult<ProductRow> {
        let created = self.repo.create(category).await?;

        Ok(created)
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<ProductRow> {
        let res = self.repo.get_by_id(id).await?;
        Ok(res)
    }

    async fn update(&self, id: i64, category: UpdateProduct) -> ApiResult<ProductRow> {
        let updated = self.repo.update(id, category).await?;

        Ok(updated)
    }

    async fn delete(&self, id: i64) -> ApiResult<()> {
        Ok(self.repo.delete(id).await?)
    }
}
