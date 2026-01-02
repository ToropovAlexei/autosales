use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::RepositoryResult,
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        common::{ListQuery, PaginatedResult},
        stock_movement::{NewStockMovement, StockMovementRow, StockMovementType},
    },
};

#[async_trait]
pub trait StockMovementRepositoryTrait {
    async fn get_list(
        &self,
        query: ListQuery,
    ) -> RepositoryResult<PaginatedResult<StockMovementRow>>;
    async fn create(&self, stock_movement: NewStockMovement) -> RepositoryResult<StockMovementRow>;
}

#[derive(Clone)]
pub struct StockMovementRepository {
    pool: Arc<PgPool>,
}

impl StockMovementRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StockMovementRepositoryTrait for StockMovementRepository {
    async fn get_list(
        &self,
        query: ListQuery,
    ) -> RepositoryResult<PaginatedResult<StockMovementRow>> {
        let mut count_qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM stock_movements");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
        SELECT
            id, order_id, product_id, type as "type: _", 
            quantity, created_by, source, description, reference_id,
            balance_after, created_at
        FROM stock_movements"#,
        );
        apply_list_query(&mut query_builder, &query);
        let query = query_builder.build_query_as::<StockMovementRow>();
        let items = query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn create(&self, stock_movement: NewStockMovement) -> RepositoryResult<StockMovementRow> {
        let result = sqlx::query_as!(
            StockMovementRow,
            r#"
            INSERT INTO stock_movements (order_id, product_id, type, quantity, created_by, source, description, reference_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING 
                id, order_id, product_id, type as "type: _", 
                quantity, created_by, source, description, reference_id,
                balance_after, created_at
            "#,
            stock_movement.order_id,
            stock_movement.product_id,
            stock_movement.r#type as StockMovementType,
            stock_movement.quantity,
            stock_movement.created_by,
            stock_movement.source,
            stock_movement.description,
            stock_movement.reference_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }
}
