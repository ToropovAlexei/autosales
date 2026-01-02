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
        sqlx::query!("UPDATE images SET deleted_at = NOW() WHERE id = $1", id)
            .execute(&*self.pool)
            .await?;
        Ok(())
    }
}
