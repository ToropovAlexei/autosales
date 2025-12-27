use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::category::{CategoryRepository, CategoryRepositoryTrait},
    models::category::{CategoryRow, NewCategory, UpdateCategory},
};

#[async_trait]
pub trait CategoryServiceTrait: Send + Sync {
    async fn get_list(&self) -> ApiResult<Vec<CategoryRow>>;
    async fn create(&self, category: NewCategory) -> ApiResult<CategoryRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<CategoryRow>;
    async fn update(&self, id: i64, category: UpdateCategory) -> ApiResult<CategoryRow>;
    async fn delete(&self, id: i64) -> ApiResult<()>;
}

pub struct CategoryService<R> {
    repo: Arc<R>,
}

impl<R> CategoryService<R>
where
    R: CategoryRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl CategoryServiceTrait for CategoryService<CategoryRepository> {
    async fn get_list(&self) -> ApiResult<Vec<CategoryRow>> {
        self.repo
            .get_list()
            .await
            .map_err(|_| ApiError::InternalServerError)
    }

    async fn create(&self, category: NewCategory) -> ApiResult<CategoryRow> {
        if let Some(parent_id) = category.parent_id {
            let parent = self.repo.get_by_id(parent_id).await;

            if parent.is_err() {
                return Err(ApiError::BadRequest(
                    "Parent category does not exist".to_string(),
                ));
            }
        };

        let created = self.repo.create(category).await?;

        Ok(created)
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<CategoryRow> {
        let res = self.repo.get_by_id(id).await?;
        Ok(res)
    }

    async fn update(&self, id: i64, category: UpdateCategory) -> ApiResult<CategoryRow> {
        if let Some(ref name) = category.name
            && name.trim().is_empty()
        {
            return Err(ApiError::BadRequest(
                "Category name cannot be empty or whitespace only".to_string(),
            ));
        }

        if let Some(Some(new_parent_id)) = category.parent_id {
            if new_parent_id == id {
                return Err(ApiError::BadRequest(
                    "Cannot set parent to self".to_string(),
                ));
            }
            let parent = self.repo.get_by_id(new_parent_id).await;

            if parent.is_err() {
                return Err(ApiError::BadRequest(
                    "Parent category does not exist".to_string(),
                ));
            }
        }

        let updated = self.repo.update(id, category).await?;

        Ok(updated)
    }

    async fn delete(&self, id: i64) -> ApiResult<()> {
        if !self.repo.get_by_parent_id(id).await?.is_empty() {
            return Err(ApiError::BadRequest(
                "Cannot delete category with child categories".to_string(),
            ));
        }

        Ok(self.repo.delete(id).await?)
    }
}
