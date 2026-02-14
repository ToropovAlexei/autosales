use std::sync::Arc;

use async_trait::async_trait;
use shared_dtos::product::ProductType;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::{RepositoryError, RepositoryResult},
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::PaginatedResult,
        product::{NewProduct, ProductListQuery, ProductRow, UpdateProduct},
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
    async fn get_for_external_provider(
        &self,
        provider_name: &str,
        external_id: &str,
    ) -> RepositoryResult<ProductRow>;
    async fn get_all_external_provider(
        &self,
        provider_name: &str,
    ) -> RepositoryResult<Vec<ProductRow>>;
    async fn find_by_name(&self, name: &str) -> RepositoryResult<ProductRow>;
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
                provider_name, external_id, created_by, created_at, updated_at, deleted_at,
                stock
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
                provider_name, external_id, created_by, created_at, updated_at, deleted_at,
                stock
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
                provider_name, external_id, created_by, created_at, updated_at, deleted_at,
                stock
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
            if let Some(image_id) = image_id {
                query_builder.push_bind(image_id);
            } else {
                query_builder.push("NULL");
            }
        }

        if let Some(r#type) = product.r#type {
            query_builder.push(", type = ");
            query_builder.push_bind(r#type);
        }

        if let Some(external_id) = product.external_id {
            query_builder.push(", external_id = ");
            if let Some(external_id) = external_id {
                query_builder.push_bind(external_id);
            } else {
                query_builder.push("NULL");
            }
        }

        if let Some(category_id) = product.category_id {
            query_builder.push(", category_id = ");
            query_builder.push_bind(category_id);
        }

        if let Some(details) = product.details {
            query_builder.push(", details = ");
            if let Some(details) = details {
                query_builder.push_bind(details);
            } else {
                query_builder.push("NULL");
            }
        }

        if let Some(fulfillment_text) = product.fulfillment_text {
            query_builder.push(", fulfillment_text = ");
            if let Some(fulfillment_text) = fulfillment_text {
                query_builder.push_bind(fulfillment_text);
            } else {
                query_builder.push("NULL");
            }
        }

        if let Some(fulfillment_image_id) = product.fulfillment_image_id {
            query_builder.push(", fulfillment_image_id = ");
            if let Some(fulfillment_image_id) = fulfillment_image_id {
                query_builder.push_bind(fulfillment_image_id);
            } else {
                query_builder.push("NULL");
            }
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

    async fn get_for_external_provider(
        &self,
        provider_name: &str,
        external_id: &str,
    ) -> RepositoryResult<ProductRow> {
        let result = sqlx::query_as!(
            ProductRow,
            r#"SELECT
                id, name, base_price, category_id, image_id, type as "type: _",
                subscription_period_days, details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by, created_at, updated_at, deleted_at,
                stock
            FROM products WHERE provider_name = $1 AND external_id = $2 AND deleted_at IS NULL"#,
            provider_name,
            external_id
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn get_all_external_provider(
        &self,
        provider_name: &str,
    ) -> RepositoryResult<Vec<ProductRow>> {
        let result = sqlx::query_as!(
            ProductRow,
            r#"SELECT
                id, name, base_price, category_id, image_id, type as "type: _",
                subscription_period_days, details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by, created_at, updated_at, deleted_at,
                stock
            FROM products WHERE provider_name = $1 AND deleted_at IS NULL"#,
            provider_name
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn find_by_name(&self, name: &str) -> RepositoryResult<ProductRow> {
        let result = sqlx::query_as!(
            ProductRow,
            r#"SELECT
                id, name, base_price, category_id, image_id, type as "type: _",
                subscription_period_days, details, fulfillment_text, fulfillment_image_id,
                provider_name, external_id, created_by, created_at, updated_at, deleted_at,
                stock
            FROM products WHERE name = $1 AND deleted_at IS NULL"#,
            name
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::RngExt;
    use rust_decimal::Decimal;
    use shared_dtos::list_query::Pagination;
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
                provider_name, external_id, created_by, created_at, updated_at, deleted_at,
                stock
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
                provider_name, external_id, created_by, created_at, updated_at, deleted_at,
                stock
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

    #[sqlx::test]
    async fn test_create_and_get_product(pool: PgPool) {
        let repo = ProductRepository::new(Arc::new(pool.clone()));
        let category_id = create_test_category(&pool, "Electronics").await;

        let new_product = NewProduct {
            name: "Test Product".to_string(),
            base_price: Decimal::try_from(100.00).unwrap(),
            category_id,
            image_id: None,
            r#type: ProductType::Item,
            subscription_period_days: 0,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            provider_name: "test_provider".to_string(),
            external_id: None,
            created_by: 1,
        };

        let created_product = repo.create(new_product.clone()).await.unwrap();
        assert_eq!(created_product.name, new_product.name);
        assert_eq!(created_product.base_price, new_product.base_price);

        let fetched_product = repo.get_by_id(created_product.id).await.unwrap();
        assert_eq!(fetched_product.id, created_product.id);
        assert_eq!(fetched_product.name, new_product.name);
    }

    #[sqlx::test]
    async fn test_delete_product(pool: PgPool) {
        let repo = ProductRepository::new(Arc::new(pool.clone()));
        let category_id = create_test_category(&pool, "Deletable").await;
        let product_to_delete =
            create_test_product(&pool, "To Be Deleted", 50.0, category_id).await;

        repo.delete(product_to_delete.id).await.unwrap();

        let result = repo.get_by_id(product_to_delete.id).await;
        assert!(matches!(result, Err(RepositoryError::NotFound(_))));
    }

    #[sqlx::test]
    async fn test_get_by_id_not_found(pool: PgPool) {
        let repo = ProductRepository::new(Arc::new(pool.clone()));
        let non_existent_id = -1;

        let result = repo.get_by_id(non_existent_id).await;
        assert!(matches!(result, Err(RepositoryError::NotFound(_))));
    }

    #[sqlx::test]
    async fn test_get_list_products(pool: PgPool) {
        let repo = ProductRepository::new(Arc::new(pool.clone()));
        let category_id = create_test_category(&pool, "ListCategory").await;

        // Create several products
        create_test_product(&pool, "Product A", 10.0, category_id).await;
        create_test_product(&pool, "Product B", 20.0, category_id).await;
        create_test_product(&pool, "Product C", 30.0, category_id).await;

        let query = ProductListQuery {
            pagination: Pagination {
                page: 1,
                page_size: 2,
            },
            ..Default::default()
        };

        let result = repo.get_list(query).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.total >= 3); // Account for other tests creating products
    }

    #[sqlx::test]
    async fn test_get_for_external_provider(pool: PgPool) {
        let repo = ProductRepository::new(Arc::new(pool.clone()));
        let category_id = create_test_category(&pool, "ExternalProviderCategory").await;

        let provider_name = "external_shop";
        let external_id = "ext_prod_123";

        let new_product = NewProduct {
            name: "External Product".to_string(),
            base_price: Decimal::new(99, 2),
            category_id,
            image_id: None,
            r#type: ProductType::Item,
            subscription_period_days: 0,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            provider_name: provider_name.to_string(),
            external_id: Some(external_id.to_string()),
            created_by: 1,
        };

        repo.create(new_product.clone()).await.unwrap();

        let fetched_product = repo
            .get_for_external_provider(provider_name, external_id)
            .await
            .unwrap();
        assert_eq!(fetched_product.name, new_product.name);
        assert_eq!(fetched_product.provider_name, provider_name);
        assert_eq!(fetched_product.external_id.unwrap(), external_id);

        let non_existent_result = repo.get_for_external_provider("non_existent", "id").await;
        assert!(matches!(
            non_existent_result,
            Err(RepositoryError::NotFound(_))
        ));
    }

    #[sqlx::test]
    async fn test_get_all_external_provider(pool: PgPool) {
        let repo = ProductRepository::new(Arc::new(pool.clone()));
        let category_id = create_test_category(&pool, "AllExternalProviderCategory").await;

        let provider_name = "another_shop";
        create_test_product(&pool, "Prod 1", 100.0, category_id).await; // created by default helper
        create_test_product(&pool, "Prod 2", 200.0, category_id).await; // created by default helper

        // Manually create products with the specific provider_name
        let new_product1 = NewProduct {
            name: "Manual Prod 1".to_string(),
            base_price: Decimal::new(100, 2),
            category_id,
            image_id: None,
            r#type: ProductType::Item,
            subscription_period_days: 0,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            provider_name: provider_name.to_string(),
            external_id: Some("manual_ext_1".to_string()),
            created_by: 1,
        };
        let new_product2 = NewProduct {
            name: "Manual Prod 2".to_string(),
            base_price: Decimal::new(200, 2),
            category_id,
            image_id: None,
            r#type: ProductType::Item,
            subscription_period_days: 0,
            details: None,
            fulfillment_text: None,
            fulfillment_image_id: None,
            provider_name: provider_name.to_string(),
            external_id: Some("manual_ext_2".to_string()),
            created_by: 1,
        };
        repo.create(new_product1).await.unwrap();
        repo.create(new_product2).await.unwrap();

        let products = repo.get_all_external_provider(provider_name).await.unwrap();
        assert_eq!(products.len(), 2);
        assert!(
            products
                .iter()
                .any(|p| p.name == "Manual Prod 1" && p.provider_name == provider_name)
        );
        assert!(
            products
                .iter()
                .any(|p| p.name == "Manual Prod 2" && p.provider_name == provider_name)
        );

        let empty_products = repo
            .get_all_external_provider("non_existent_provider")
            .await
            .unwrap();
        assert!(empty_products.is_empty());
    }

    #[sqlx::test]
    async fn test_find_by_name(pool: PgPool) {
        let repo = ProductRepository::new(Arc::new(pool.clone()));
        let category_id = create_test_category(&pool, "FindByNameCategory").await;

        let product_name = "Unique Product Name";
        create_test_product(&pool, product_name, 100.0, category_id).await;

        let fetched_product = repo.find_by_name(product_name).await.unwrap();
        assert_eq!(fetched_product.name, product_name);

        let non_existent_result = repo.find_by_name("Non Existent Name").await;
        assert!(matches!(
            non_existent_result,
            Err(RepositoryError::NotFound(_))
        ));
    }
}
