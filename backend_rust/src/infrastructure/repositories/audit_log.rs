use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::{
    errors::repository::RepositoryResult,
    infrastructure::lib::query::{apply_filters, apply_list_query},
    models::{
        audit_log::{AuditLogListQuery, AuditLogRow, NewAuditLog},
        common::PaginatedResult,
    },
};

#[async_trait]
pub trait AuditLogRepositoryTrait {
    async fn get_list(
        &self,
        query: AuditLogListQuery,
    ) -> RepositoryResult<PaginatedResult<AuditLogRow>>;
    async fn create(&self, audit_log: NewAuditLog) -> RepositoryResult<AuditLogRow>;
}

#[derive(Clone)]
pub struct AuditLogRepository {
    pool: Arc<PgPool>,
}

impl AuditLogRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditLogRepositoryTrait for AuditLogRepository {
    async fn get_list(
        &self,
        query: AuditLogListQuery,
    ) -> RepositoryResult<PaginatedResult<AuditLogRow>> {
        let mut count_qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM audit_logs");
        apply_filters(&mut count_qb, &query);

        let count_query = count_qb.build_query_scalar();
        let total: i64 = count_query.fetch_one(&*self.pool).await?;

        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM audit_logs");
        apply_list_query(&mut query_builder, &query);
        let query = query_builder.build_query_as::<AuditLogRow>();
        let items = query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn create(&self, audit_log: NewAuditLog) -> RepositoryResult<AuditLogRow> {
        let result = sqlx::query_as!(
            AuditLogRow,
            r#"
            INSERT INTO audit_logs (
                admin_user_id, customer_id, action, status, target_table, target_id, 
                old_values, new_values, ip_address, user_agent, request_id, error_message
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING
                id, admin_user_id, customer_id, action as "action: _", status as "status: _", target_table, target_id, 
                old_values, new_values, ip_address, user_agent, request_id, error_message, created_at
            "#,
            audit_log.admin_user_id,
            audit_log.customer_id,
            audit_log.action as _,
            audit_log.status as _,
            audit_log.target_table,
            audit_log.target_id,
            audit_log.old_values,
            audit_log.new_values,
            audit_log.ip_address,
            audit_log.user_agent,
            audit_log.request_id,
            audit_log.error_message
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }
}
