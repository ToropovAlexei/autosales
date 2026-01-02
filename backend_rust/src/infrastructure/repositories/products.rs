use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::{RepositoryError, RepositoryResult},
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        product::{NewProduct, ProductListQuery, ProductRow, ProductType, UpdateProduct},
    },
};

#[async_trait]
pub trait ProductRepositoryTrait {
    async fn get_list(
        &self,
        query: ProductListQuery,
    ) -> RepositoryResult<PaginatedResult<ProductRow>>;
    async fn create(&self, product: NewProduct) -> RepositoryResult<ProductRow>;
    async fn get_by_id(&self, id: i64) -> RepositoryResult<ProductRow>;
    async fn update(&self, id: i64, product: UpdateProduct) -> RepositoryResult<ProductRow>;
    async fn delete(&self, id: i64) -> RepositoryResult<()>;
}

#[derive(Clone)]
pub struct ProductRepository {
    pool: Arc<PgPool>,
}

impl ProductRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepositoryTrait for ProductRepository {
    async fn get_list(
        &self,
        query: ProductListQuery,
    ) -> RepositoryResult<PaginatedResult<ProductRow>> {
        let mut count_qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM products");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"SELECT
                id, name, price, category_id, image_id, type as "type: _",
                subscription_period_days, details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by, created_at, updated_at, deleted_at
            FROM products"#,
        );
        apply_list_query(&mut query_builder, &query);
        let query = query_builder.build_query_as::<ProductRow>();
        let items = query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn create(&self, product: NewProduct) -> RepositoryResult<ProductRow> {
        let result = sqlx::query_as!(
            ProductRow,
            r#"
            INSERT INTO products (
                name, price, category_id, image_id, type, subscription_period_days,
                details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, name, price, category_id, image_id, type as "type: _",
                subscription_period_days, details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by, created_at, updated_at, deleted_at
            "#,
            product.name,
            product.price,
            product.category_id,
            product.image_id,
            product.r#type as ProductType,
            product.subscription_period_days,
            product.details,
            product.fulfillment_text,
            product.fulfillment_image_id,
            product.provider_name,
            product.external_id,
            product.created_by,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn get_by_id(&self, id: i64) -> RepositoryResult<ProductRow> {
        let result = sqlx::query_as!(
            ProductRow,
            r#"SELECT
                id, name, price, category_id, image_id, type as "type: _",
                subscription_period_days, details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by, created_at, updated_at, deleted_at
            FROM products WHERE id = $1 AND deleted_at IS NULL"#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn update(&self, id: i64, product: UpdateProduct) -> RepositoryResult<ProductRow> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("UPDATE products SET name = COALESCE(");

        query_builder.push_bind(product.name);
        query_builder.push(", name)");

        if let Some(image_id) = product.image_id {
            query_builder.push(", image_id = ");
            query_builder.push_bind(image_id);
        }

        if let Some(r#type) = product.r#type {
            query_builder.push(", type = ");
            query_builder.push_bind(r#type);
        }

        if let Some(external_id) = product.external_id {
            query_builder.push(", external_id = ");
            query_builder.push_bind(external_id);
        }

        if let Some(category_id) = product.category_id {
            query_builder.push(", category_id = ");
            query_builder.push_bind(category_id);
        }

        if let Some(details) = product.details {
            query_builder.push(", details = ");
            query_builder.push_bind(details);
        }

        if let Some(fulfillment_text) = product.fulfillment_text {
            query_builder.push(", fulfillment_text = ");
            query_builder.push_bind(fulfillment_text);
        }

        if let Some(fulfillment_image_id) = product.fulfillment_image_id {
            query_builder.push(", fulfillment_image_id = ");
            query_builder.push_bind(fulfillment_image_id);
        }

        if let Some(price) = product.price {
            query_builder.push(", price = ");
            query_builder.push_bind(price);
        }

        if let Some(subscription_period_days) = product.subscription_period_days {
            query_builder.push(", subscription_period_days = ");
            query_builder.push_bind(subscription_period_days);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push("AND deleted_at IS NULL RETURNING *");

        let query = query_builder.build_query_as::<ProductRow>();

        query
            .fetch_one(&*self.pool)
            .await
            .map_err(RepositoryError::from)
    }

    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query!(
            "UPDATE products SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}
