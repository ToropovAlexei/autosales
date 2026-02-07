use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::{
    errors::repository::RepositoryResult,
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        image::{ImageListQuery, ImageRow, NewImage},
    },
};

#[async_trait]
pub trait ImageRepositoryTrait {
    async fn get_list(&self, query: &ImageListQuery)
    -> RepositoryResult<PaginatedResult<ImageRow>>;
    async fn get_by_id(&self, id: Uuid) -> RepositoryResult<ImageRow>;
    async fn create(&self, image: NewImage) -> RepositoryResult<ImageRow>;
    async fn delete(&self, id: Uuid) -> RepositoryResult<()>;
    async fn get_by_hash(&self, hash: String) -> RepositoryResult<ImageRow>;
}

#[derive(Clone)]
pub struct ImageRepository {
    pool: Arc<PgPool>,
}

impl ImageRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ImageRepositoryTrait for ImageRepository {
    async fn get_list(
        &self,
        query: &ImageListQuery,
    ) -> RepositoryResult<PaginatedResult<ImageRow>> {
        let mut count_qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT COUNT(*) FROM images");
        apply_filters(&mut count_qb, query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM images");
        apply_list_query(&mut query_builder, query);
        let items_query = query_builder.build_query_as::<ImageRow>();
        let items = items_query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn get_by_id(&self, id: Uuid) -> RepositoryResult<ImageRow> {
        let result = sqlx::query_as!(ImageRow, "SELECT * FROM images WHERE id = $1", id)
            .fetch_one(&*self.pool)
            .await?;
        Ok(result)
    }

    async fn create(&self, image: NewImage) -> RepositoryResult<ImageRow> {
        let result = sqlx::query_as!(
            ImageRow,
            r#"
            INSERT INTO images (original_filename, hash, mime_type, file_size, width, height, context, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            image.original_filename,
            image.hash,
            image.mime_type,
            image.file_size,
            image.width,
            image.height,
            image.context,
            image.created_by
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn delete(&self, id: Uuid) -> RepositoryResult<()> {
        sqlx::query!("DELETE FROM images WHERE id = $1", id)
            .execute(&*self.pool)
            .await?;
        Ok(())
    }

    async fn get_by_hash(&self, hash: String) -> RepositoryResult<ImageRow> {
        let result = sqlx::query_as!(ImageRow, "SELECT * FROM images WHERE hash = $1", hash)
            .fetch_one(&*self.pool)
            .await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn create_test_image(pool: &PgPool, original_filename: &str) -> ImageRow {
        let new_image = NewImage {
            original_filename: Some(original_filename.to_string()),
            hash: original_filename.to_string(),
            mime_type: "image/png".to_string(),
            file_size: 100,
            width: 10_i16,
            height: 10_i16,
            context: "product".to_string(),
            created_by: 1,
        };
        sqlx::query_as!(
            ImageRow,
            r#"
            INSERT INTO images (original_filename, hash, mime_type, file_size, width, height, context, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            new_image.original_filename,
            new_image.hash,
            new_image.mime_type,
            new_image.file_size,
            new_image.width,
            new_image.height,
            new_image.context,
            new_image.created_by
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_get_image(pool: PgPool) {
        let repo = ImageRepository::new(Arc::new(pool.clone()));
        let filename = "test_image.png";

        // Create an image
        let created_image = create_test_image(&pool, filename).await;
        assert_eq!(created_image.original_filename, Some(filename.to_string()));

        // Get by id
        let fetched_image = repo.get_by_id(created_image.id).await.unwrap();
        assert_eq!(fetched_image.id, created_image.id);
    }

    #[sqlx::test]
    async fn test_delete_image(pool: PgPool) {
        let repo = ImageRepository::new(Arc::new(pool.clone()));
        let filename = "image_to_delete.png";

        // Create an image
        let image = create_test_image(&pool, filename).await;

        // Delete the image
        repo.delete(image.id).await.unwrap();

        // Try to get the image again
        let result = repo.get_by_id(image.id).await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_get_list_images(pool: PgPool) {
        let repo = ImageRepository::new(Arc::new(pool.clone()));

        // Create some images
        create_test_image(&pool, "image1.png").await;
        create_test_image(&pool, "image2.png").await;

        // Get the list of images
        let query = ImageListQuery::default();
        let images = repo.get_list(&query).await.unwrap();
        assert!(!images.items.is_empty());
        assert!(images.total >= 2);
    }
}
