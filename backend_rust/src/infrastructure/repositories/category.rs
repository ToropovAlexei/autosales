use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::{RepositoryError, RepositoryResult},
    models::category::{CategoryRow, NewCategory, UpdateCategory},
};

#[async_trait]
pub trait CategoryRepositoryTrait {
    async fn get_list(&self) -> RepositoryResult<Vec<CategoryRow>>;
    async fn create(&self, category: NewCategory) -> RepositoryResult<CategoryRow>;
    async fn get_by_id(&self, id: i64) -> RepositoryResult<Option<CategoryRow>>;
    async fn update(&self, id: i64, category: UpdateCategory) -> RepositoryResult<CategoryRow>;
    async fn delete(&self, id: i64) -> RepositoryResult<()>;
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
    async fn get_list(&self) -> RepositoryResult<Vec<CategoryRow>> {
        let result = sqlx::query_as!(
            CategoryRow,
            "SELECT * FROM categories ORDER BY position ASC"
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn create(&self, category: NewCategory) -> RepositoryResult<CategoryRow> {
        let result = sqlx::query_as!(
            CategoryRow,
            r#"
            INSERT INTO categories (name, parent_id, image_id, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            category.name,
            category.parent_id,
            category.image_id,
            category.created_by
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn get_by_id(&self, id: i64) -> RepositoryResult<Option<CategoryRow>> {
        let result = sqlx::query_as!(CategoryRow, "SELECT * FROM categories WHERE id = $1", id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(result)
    }

    async fn update(&self, id: i64, category: UpdateCategory) -> RepositoryResult<CategoryRow> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE categories SET ");

        if let Some(name) = category.name {
            query_builder.push(", name = ");
            query_builder.push_bind(name);
        }

        if let Some(parent_id) = category.parent_id {
            query_builder.push(", parent_id = ");
            query_builder.push_bind(parent_id);
        }

        if let Some(image_id) = category.image_id {
            query_builder.push(", image_id = ");
            query_builder.push_bind(image_id);
        }

        if let Some(position) = category.position {
            query_builder.push(", position = ");
            query_builder.push_bind(position);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" RETURNING *");

        let query = query_builder.build_query_as::<CategoryRow>();

        query
            .fetch_one(&*self.pool)
            .await
            .map_err(RepositoryError::from)
    }

    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query!("DELETE FROM categories WHERE id = $1", id)
            .execute(&*self.pool)
            .await?;
        Ok(())
    }
}
