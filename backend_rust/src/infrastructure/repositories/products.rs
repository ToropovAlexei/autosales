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
                id, name, base_price, category_id, image_id, type,
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
                name, base_price, category_id, image_id, type, subscription_period_days,
                details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, name, base_price, category_id, image_id, type as "type: _",
                subscription_period_days, details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by, created_at, updated_at, deleted_at
            "#,
            product.name,
            product.base_price,
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
                id, name, base_price, category_id, image_id, type as "type: _",
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

        if let Some(base_price) = product.base_price {
            query_builder.push(", base_price = ");
            query_builder.push_bind(base_price);
        }

        if let Some(subscription_period_days) = product.subscription_period_days {
            query_builder.push(", subscription_period_days = ");
            query_builder.push_bind(subscription_period_days);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" AND deleted_at IS NULL RETURNING *");

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::product::ProductType;
    use rand::Rng;
    use rust_decimal::Decimal;
    use sqlx::PgPool;
    use uuid::Uuid;

    // Helper to create a product for tests
    async fn create_test_product(
        pool: &PgPool,
        name: &str,
        base_price: f64,
        category_id: i64,
    ) -> ProductRow {
        sqlx::query_as!(
            ProductRow,
            r#"
            INSERT INTO products (name, base_price, category_id, type, subscription_period_days, provider_name, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, base_price, category_id, image_id, type as "type: _",
                subscription_period_days, details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by, created_at, updated_at, deleted_at
            "#,
            name,
            Decimal::try_from(base_price).unwrap(),
            category_id,
            ProductType::Item as ProductType, // Default type
            0, // Default subscription_period_days
            "test_provider", // Default provider
            1 // Default created_by
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    // Helper to create a category for testing
    async fn create_test_category(pool: &PgPool, name: &str) -> i64 {
        sqlx::query!(
            "INSERT INTO categories (name, created_by) VALUES ($1, $2) RETURNING id",
            name,
            1
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id
    }

    pub fn generate_random_hash() -> String {
        let mut rng = rand::rng();
        let chars = "abcdefghijklmnopqrstuvwxyz";
        let password: String = (0..4)
            .map(|_| {
                let idx = rng.random_range(0..chars.len());
                chars.chars().nth(idx).unwrap()
            })
            .collect();
        password
    }

    async fn create_test_image(pool: &PgPool) -> Uuid {
        sqlx::query!(
            r#"
            INSERT INTO images (original_filename, hash, mime_type, file_size, width, height, context, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            "test_image.jpg",
            generate_random_hash(),
            "image/jpeg",
            1000,
            800,
            600,
            "product",
            1
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id
    }

    #[sqlx::test]
    async fn test_update_product_all_some(pool: PgPool) {
        let repo = ProductRepository::new(Arc::new(pool.clone()));
        let category_id = create_test_category(&pool, "Electronics").await;

        // Create a product
        let initial_product = create_test_product(&pool, "Laptop", 1200.00, category_id).await;
        let new_image_id = create_test_image(&pool).await;
        let new_fulfillment_image_id = create_test_image(&pool).await;

        let update_data = UpdateProduct {
            name: Some("Gaming Laptop".to_string()),
            base_price: Some(Decimal::try_from(1500.00).unwrap()),
            category_id: Some(category_id),
            image_id: Some(Some(new_image_id)),
            r#type: Some(ProductType::Subscription),
            subscription_period_days: Some(30),
            details: Some(Some(serde_json::json!({"cpu": "i9"}))),
            fulfillment_text: Some(Some("Digital code via email".to_string())),
            fulfillment_image_id: Some(Some(new_fulfillment_image_id)),
            external_id: Some(Some("EXT123".to_string())),
        };

        let _updated_product = repo.update(initial_product.id, update_data).await.unwrap();

        // Fetch the product again to verify
        let fetched_product = repo.get_by_id(initial_product.id).await.unwrap();

        assert_eq!(fetched_product.id, initial_product.id);
        assert_eq!(fetched_product.name, "Gaming Laptop");
        assert_eq!(
            fetched_product.base_price,
            Decimal::try_from(1500.00).unwrap()
        );
        assert_eq!(fetched_product.r#type, ProductType::Subscription);
        assert_eq!(fetched_product.subscription_period_days, 30);
        assert_eq!(fetched_product.details.unwrap()["cpu"], "i9");
        assert_eq!(
            fetched_product.fulfillment_text.unwrap(),
            "Digital code via email"
        );
        assert_eq!(
            fetched_product.fulfillment_image_id.unwrap(),
            new_fulfillment_image_id
        );
        assert_eq!(fetched_product.external_id.unwrap(), "EXT123");
        assert_eq!(fetched_product.image_id.unwrap(), new_image_id);
    }

    #[sqlx::test]
    async fn test_update_product_some_none_mix(pool: PgPool) {
        let repo = ProductRepository::new(Arc::new(pool.clone()));
        let category_id = create_test_category(&pool, "Books").await;

        // Create a product with some initial optional values
        let initial_image_id = create_test_image(&pool).await;
        let initial_fulfillment_image_id = create_test_image(&pool).await;
        let initial_product = sqlx::query_as!(
            ProductRow,
            r#"
            INSERT INTO products (name, base_price, category_id, image_id, type, subscription_period_days, details, fulfillment_image_id, external_id, provider_name, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, name, base_price, category_id, image_id, type as "type: _",
                subscription_period_days, details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by, created_at, updated_at, deleted_at
            "#,
            "Old Book",
            Decimal::try_from(25.00).unwrap(),
            category_id,
            Some(initial_image_id),
            ProductType::Item as ProductType,
            0,
            Some(serde_json::json!({"author": "Unknown"})),
            Some(initial_fulfillment_image_id),
            Some("OLDEXT".to_string()),
            "another_provider",
            1
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        // Update some fields with Some(value), some with Some(None) (to nullify), some with None (to keep original)
        let update_data = UpdateProduct {
            name: Some("Newer Book".to_string()), // Update
            base_price: None,                     // Keep original
            category_id: None,                    // Keep original
            image_id: Some(None),                 // Set to NULL
            r#type: None,                         // Keep original
            subscription_period_days: Some(0),    // Update
            details: Some(None),                  // Set to NULL
            fulfillment_text: Some(Some("Download link".to_string())), // Update
            fulfillment_image_id: None,           // Keep original
            external_id: None,                    // Keep original
        };

        let _updated_product = repo.update(initial_product.id, update_data).await.unwrap();

        // Fetch the product again to verify
        let fetched_product = repo.get_by_id(initial_product.id).await.unwrap();

        assert_eq!(fetched_product.id, initial_product.id);
        assert_eq!(fetched_product.name, "Newer Book");
        assert_eq!(fetched_product.base_price, initial_product.base_price); // Unchanged
        assert_eq!(fetched_product.category_id, initial_product.category_id); // Unchanged
        assert!(fetched_product.image_id.is_none()); // Set to NULL
        assert_eq!(fetched_product.r#type, initial_product.r#type); // Unchanged
        assert_eq!(fetched_product.subscription_period_days, 0); // Updated
        assert!(fetched_product.details.is_none()); // Set to NULL
        assert_eq!(fetched_product.fulfillment_text.unwrap(), "Download link"); // Updated
        assert_eq!(
            fetched_product.fulfillment_image_id,
            initial_product.fulfillment_image_id
        ); // Unchanged
        assert_eq!(fetched_product.external_id, initial_product.external_id); // Unchanged
    }

    #[sqlx::test]
    async fn test_update_product_no_updates(pool: PgPool) {
        let repo = ProductRepository::new(Arc::new(pool.clone()));
        let category_id = create_test_category(&pool, "Gizmos").await;

        // Create a product
        let initial_product = create_test_product(&pool, "Widget", 10.00, category_id).await;

        // Update with all None values
        let update_data = UpdateProduct {
            name: None,
            base_price: None,
            category_id: None,
            image_id: None,
            r#type: None,
            subscription_period_days: None,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            external_id: None,
        };

        let _updated_product = repo.update(initial_product.id, update_data).await.unwrap();

        // Fetch the product again to verify
        let fetched_product = repo.get_by_id(initial_product.id).await.unwrap();

        assert_eq!(fetched_product.id, initial_product.id);
        assert_eq!(fetched_product.name, initial_product.name);
        assert_eq!(fetched_product.base_price, initial_product.base_price);
        assert_eq!(fetched_product.category_id, initial_product.category_id);
        assert_eq!(fetched_product.image_id, initial_product.image_id);
        assert_eq!(fetched_product.r#type, initial_product.r#type);
        assert_eq!(
            fetched_product.subscription_period_days,
            initial_product.subscription_period_days
        );
        assert_eq!(fetched_product.details, initial_product.details);
        assert_eq!(
            fetched_product.fulfillment_text,
            initial_product.fulfillment_text
        );
        assert_eq!(
            fetched_product.fulfillment_image_id,
            initial_product.fulfillment_image_id
        );
        assert_eq!(fetched_product.external_id, initial_product.external_id);
        // Only updated_at should change
        assert_ne!(fetched_product.updated_at, initial_product.updated_at);
    }
}
