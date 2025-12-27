use std::sync::Arc;

use crate::{
    api::dto::category::Category,
    errors::ServiceError,
    models::category::CategoryEntity,
    repositories::category::{CategoryRepositoryTrait, NewCategory, UpdateCategory},
};

type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Clone)]
pub struct CategoryService {
    repo: Arc<dyn CategoryRepositoryTrait>,
}

impl CategoryService {
    pub fn new(repo: Arc<dyn CategoryRepositoryTrait>) -> Self {
        Self { repo }
    }

    pub async fn list_all(&self) -> ServiceResult<Vec<Category>> {
        let entities = self.repo.list_all().await?;
        let dtos = entities.into_iter().map(Category::from).collect();
        Ok(dtos)
    }

    pub async fn get_by_id(&self, id: i64) -> ServiceResult<Category> {
        let entity = self.repo.get_by_id(id).await?;
        Ok(Category::from(entity))
    }

    pub async fn create(&self, data: NewCategory) -> ServiceResult<Category> {
        // Business logic can be added here.
        // For example, checking if a category with the same name already exists.
        let entity = self.repo.create(data).await?;
        Ok(Category::from(entity))
    }

    pub async fn update(&self, id: i64, data: UpdateCategory) -> ServiceResult<Category> {
        // Business logic for update
        let entity = self.repo.update(id, data).await?;
        Ok(Category::from(entity))
    }

    pub async fn delete(&self, id: i64) -> ServiceResult<()> {
        self.repo.delete(id).await?;
        Ok(())
    }
}

// This is the conversion from the database Entity to the API DTO.
impl From<CategoryEntity> for Category {
    fn from(entity: CategoryEntity) -> Self {
        Category {
            id: entity.id,
            name: entity.name,
            parent_id: entity.parent_id,
            image_id: entity.image_id,
            position: entity.position,
            is_active: entity.is_active,
            created_by: entity.created_by,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
