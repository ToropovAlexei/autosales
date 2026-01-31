use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::stock_movement::{
        StockMovementRepository, StockMovementRepositoryTrait,
    },
    models::{
        common::PaginatedResult,
        stock_movement::{StockMovementListQuery, StockMovementRow},
    },
};

#[async_trait]
pub trait StockMovementServiceTrait: Send + Sync {
    async fn get_list(
        &self,
        query: StockMovementListQuery,
    ) -> ApiResult<PaginatedResult<StockMovementRow>>;
}

pub struct StockMovementService<S> {
    stock_movement_repo: Arc<S>,
}

impl<S> StockMovementService<S>
where
    S: StockMovementRepositoryTrait + Send + Sync,
{
    pub fn new(stock_movement_repo: Arc<S>) -> Self {
        Self {
            stock_movement_repo,
        }
    }
}

#[async_trait]
impl StockMovementServiceTrait for StockMovementService<StockMovementRepository> {
    async fn get_list(
        &self,
        query: StockMovementListQuery,
    ) -> ApiResult<PaginatedResult<StockMovementRow>> {
        self.stock_movement_repo
            .get_list(query)
            .await
            .map_err(ApiError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::stock_movement::StockMovementRepository;
    use crate::models::stock_movement::{NewStockMovement, StockMovementType};
    use sqlx::PgPool;
    use std::sync::Arc;

    async fn create_product(pool: &PgPool, name: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO products (name, base_price, type, created_by, provider_name)
            VALUES ($1, 10.0, 'item', 1, 'test')
            RETURNING id
            "#,
            name
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    fn build_service(pool: &PgPool) -> StockMovementService<StockMovementRepository> {
        let pool = Arc::new(pool.clone());
        StockMovementService::new(Arc::new(StockMovementRepository::new(pool)))
    }

    #[sqlx::test]
    async fn test_get_list(pool: PgPool) {
        let service = build_service(&pool);
        let repo = StockMovementRepository::new(Arc::new(pool.clone()));
        let product_id = create_product(&pool, "stock_product").await;

        repo.create(NewStockMovement {
            order_id: None,
            product_id,
            r#type: StockMovementType::Restock,
            quantity: 10,
            created_by: 1,
            description: Some("restock".to_string()),
            reference_id: None,
        })
        .await
        .unwrap();

        let result = service
            .get_list(StockMovementListQuery::default())
            .await
            .unwrap();
        assert!(result.total >= 1);
        assert!(result.items.iter().any(|row| row.product_id == product_id));
    }
}
