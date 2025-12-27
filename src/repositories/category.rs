use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    errors::RepositoryError,
    models::category::CategoryEntity,
};

type RepositoryResult<T> = Result<T, RepositoryError>;

/// Input data for creating a new category.
#[derive(Debug, Clone)]
pub struct NewCategory {
    pub name: String,
    pub parent_id: Option<i64>,
    pub image_id: Option<uuid::Uuid>,
    pub created_by: i64,
}

/// Input data for updating an existing category.
#[derive(Debug, Clone, Default)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub parent_id: Option<Option<i64>>,
    pub image_id: Option<Option<uuid::Uuid>>,
    pub position: Option<i16>,
    pub is_active: Option<bool>,
}

#[async_trait]
pub trait CategoryRepositoryTrait: Send + Sync {
    async fn list_all(&self) -> RepositoryResult<Vec<CategoryEntity>>;
    async fn create(&self, data: NewCategory) -> RepositoryResult<CategoryEntity>;
    async fn update(&self, id: i64, data: UpdateCategory) -> RepositoryResult<CategoryEntity>;
    async fn delete(&self, id: i64) -> RepositoryResult<()>;
    async fn get_by_id(&self, id: i64) -> RepositoryResult<CategoryEntity>;
}

#[derive(Clone)]
pub struct CategoryRepository {
    pool: Arc<PgPool>,
}

impl CategoryRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CategoryRepositoryTrait for CategoryRepository {
    async fn list_all(&self) -> RepositoryResult<Vec<CategoryEntity>> {
        let categories = sqlx::query_as!(CategoryEntity, "SELECT * FROM categories ORDER BY position ASC, name ASC")
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::from_sqlx_error("list_all categories", e))?;
        Ok(categories)
    }

    async fn get_by_id(&self, id: i64) -> RepositoryResult<CategoryEntity> {
        let category = sqlx::query_as!(CategoryEntity, "SELECT * FROM categories WHERE id = $1", id)
            .fetch_optional(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::from_sqlx_error("get_by_id category", e))?
            .ok_or_else(|| RepositoryError::NotFound(format!("Category with id {} not found", id)))?;
        Ok(category)
    }

    async fn create(&self, data: NewCategory) -> RepositoryResult<CategoryEntity> {
        let category = sqlx::query_as!(
            CategoryEntity,
            r#"
            INSERT INTO categories (name, parent_id, image_id, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            data.name,
            data.parent_id,
            data.image_id,
            data.created_by
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::from_sqlx_error("create category", e))?;
        Ok(category)
    }

    async fn update(&self, id: i64, data: UpdateCategory) -> RepositoryResult<CategoryEntity> {
        let old = self.get_by_id(id).await?;

        let new_name = data.name.unwrap_or(old.name);
        let new_parent_id = data.parent_id.unwrap_or(old.parent_id);
        let new_image_id = data.image_id.unwrap_or(old.image_id);
        let new_position = data.position.unwrap_or(old.position);
        let new_is_active = data.is_active.unwrap_or(old.is_active);

        let updated = sqlx::query_as!(
            CategoryEntity,
            r#"
            UPDATE categories
            SET name = $1, parent_id = $2, image_id = $3, position = $4, is_active = $5, updated_at = now()
            WHERE id = $6
            RETURNING *
            "#,
            new_name,
            new_parent_id,
            new_image_id,
            new_position,
            new_is_active,
            id
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::from_sqlx_error("update category", e))?;

        Ok(updated)
    }

    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        let result = sqlx::query!("DELETE FROM categories WHERE id = $1", id)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::from_sqlx_error("delete category", e))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!("Category with id {} not found for deletion", id)));
        }

        Ok(())
    }
}