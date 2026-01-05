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
    async fn get_by_id(&self, id: i64) -> RepositoryResult<CategoryRow>;
    async fn update(&self, id: i64, category: UpdateCategory) -> RepositoryResult<CategoryRow>;
    async fn delete(&self, id: i64) -> RepositoryResult<()>;
    async fn get_by_parent_id(&self, parent_id: i64) -> RepositoryResult<Vec<CategoryRow>>;
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

    async fn get_by_id(&self, id: i64) -> RepositoryResult<CategoryRow> {
        let result = sqlx::query_as!(CategoryRow, "SELECT * FROM categories WHERE id = $1", id)
            .fetch_one(&*self.pool)
            .await?;

        Ok(result)
    }

    async fn update(&self, id: i64, category: UpdateCategory) -> RepositoryResult<CategoryRow> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("UPDATE categories SET name = COALESCE(");

        query_builder.push_bind(category.name);
        query_builder.push(", name)");

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

    async fn get_by_parent_id(&self, parent_id: i64) -> RepositoryResult<Vec<CategoryRow>> {
        let result = sqlx::query_as!(
            CategoryRow,
            "SELECT * FROM categories WHERE parent_id = $1 ORDER BY position ASC",
            parent_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn create_test_category(
        pool: &PgPool,
        name: &str,
        parent_id: Option<i64>,
    ) -> CategoryRow {
        let new_category = NewCategory {
            name: name.to_string(),
            parent_id,
            image_id: None,
            created_by: 1,
        };
        sqlx::query_as!(
            CategoryRow,
            r#"
            INSERT INTO categories (name, parent_id, image_id, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            new_category.name,
            new_category.parent_id,
            new_category.image_id,
            new_category.created_by
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_get_category(pool: PgPool) {
        let repo = CategoryRepository::new(Arc::new(pool.clone()));
        let name = "Test Category";

        // Create a category
        let created_category = create_test_category(&pool, name, None).await;
        assert_eq!(created_category.name, name);

        // Get by id
        let fetched_category = repo.get_by_id(created_category.id).await.unwrap();
        assert_eq!(fetched_category.id, created_category.id);
    }

    #[sqlx::test]
    async fn test_update_category(pool: PgPool) {
        let repo = CategoryRepository::new(Arc::new(pool.clone()));
        let name = "Category to Update";
        let new_name = "Updated Category Name";

        // Create a category
        let category = create_test_category(&pool, name, None).await;

        // Update the category
        let update_data = UpdateCategory {
            name: Some(new_name.to_string()),
            parent_id: None,
            image_id: None,
            position: Some(10),
        };
        let updated_category = repo.update(category.id, update_data).await.unwrap();
        assert_eq!(updated_category.name, new_name);
        assert_eq!(updated_category.position, 10);

        // Verify the update
        let fetched_category = repo.get_by_id(category.id).await.unwrap();
        assert_eq!(fetched_category.name, new_name);
    }

    #[sqlx::test]
    async fn test_delete_category(pool: PgPool) {
        let repo = CategoryRepository::new(Arc::new(pool.clone()));
        let name = "Category to Delete";

        // Create a category
        let category = create_test_category(&pool, name, None).await;

        // Delete the category
        repo.delete(category.id).await.unwrap();

        // Try to get the category again
        let result = repo.get_by_id(category.id).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_get_by_parent_id(pool: PgPool) {
        let repo = CategoryRepository::new(Arc::new(pool.clone()));
        let parent_name = "Parent Category";
        let parent_category = create_test_category(&pool, parent_name, None).await;

        // Create child categories
        create_test_category(&pool, "Child 1", Some(parent_category.id)).await;
        create_test_category(&pool, "Child 2", Some(parent_category.id)).await;

        // Get children by parent_id
        let children = repo.get_by_parent_id(parent_category.id).await.unwrap();
        assert_eq!(children.len(), 2);
        assert!(
            children
                .iter()
                .all(|c| c.parent_id == Some(parent_category.id))
        );
    }

    #[sqlx::test]
    async fn test_get_list_categories(pool: PgPool) {
        let repo = CategoryRepository::new(Arc::new(pool.clone()));

        // Create some categories
        create_test_category(&pool, "Category A", None).await;
        create_test_category(&pool, "Category B", None).await;

        // Get the list of categories
        let categories = repo.get_list().await.unwrap();
        assert!(!categories.is_empty());
        assert!(categories.len() >= 2);
    }
}
