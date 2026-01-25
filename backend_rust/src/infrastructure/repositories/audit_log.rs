use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, types::ipnetwork::IpNetwork};

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

        let mut query_builder = QueryBuilder::new(
            r#"
        SELECT
            audit_logs.*,
            admin_users.login AS admin_user_login
        FROM audit_logs
        LEFT JOIN admin_users ON audit_logs.admin_user_id = admin_users.id"#,
        );
        apply_list_query(&mut query_builder, &query);
        let query = query_builder.build_query_as::<AuditLogRow>();
        let items = query.fetch_all(&*self.pool).await?;
        Ok(PaginatedResult { items, total })
    }

    async fn create(&self, audit_log: NewAuditLog) -> RepositoryResult<AuditLogRow> {
        let inserted = sqlx::query_scalar!(
            r#"
            INSERT INTO audit_logs (
                admin_user_id, customer_id, action, status, target_table, target_id, 
                old_values, new_values, ip_address, user_agent, request_id, error_message
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id
            "#,
            audit_log.admin_user_id,
            audit_log.customer_id,
            audit_log.action as _,
            audit_log.status as _,
            audit_log.target_table,
            audit_log.target_id,
            audit_log.old_values,
            audit_log.new_values,
            audit_log.ip_address.map(|s| IpNetwork::from(s)),
            audit_log.user_agent,
            audit_log.request_id.map(|s| s.to_string()),
            audit_log.error_message
        )
        .fetch_one(&*self.pool)
        .await?;

        let row = sqlx::query_as!(
            AuditLogRow,
            r#"
            SELECT
                al.id,
                al.admin_user_id,
                al.customer_id,
                al.action as "action: _",
                al.status as "status: _",
                al.target_table,
                al.target_id,
                al.old_values,
                al.new_values,
                al.ip_address,
                al.user_agent,
                al.request_id,
                al.error_message,
                al.created_at,
                au.login AS "admin_user_login?"
            FROM audit_logs al
            LEFT JOIN admin_users au ON al.admin_user_id = au.id
            WHERE al.id = $1
            "#,
            inserted
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::audit_log::{AuditAction, AuditLogListQuery, AuditStatus};
    use sqlx::PgPool;
    use std::net::IpAddr;
    use std::str::FromStr;
    use uuid::Uuid;

    #[sqlx::test]
    async fn test_create_and_get_audit_log(pool: PgPool) {
        let repo = AuditLogRepository::new(Arc::new(pool));
        let new_log = NewAuditLog {
            admin_user_id: Some(1),
            customer_id: None,
            action: AuditAction::UserLogin,
            status: AuditStatus::Success,
            target_table: "admin_users".to_string(),
            target_id: "1".to_string(),
            old_values: None,
            new_values: None,
            ip_address: Some(IpAddr::from_str("127.0.0.1").unwrap()),
            user_agent: Some("test-agent".to_string()),
            request_id: Some(Uuid::new_v4()),
            error_message: None,
        };

        // Create a log
        let created_log = repo.create(new_log).await.unwrap();
        assert_eq!(created_log.admin_user_id, Some(1));
        assert_eq!(created_log.action, AuditAction::UserLogin);

        // Get the list of logs
        let query = AuditLogListQuery::default();
        let logs = repo.get_list(query).await.unwrap();
        assert!(!logs.items.is_empty());
        assert_eq!(logs.items[0].id, created_log.id);
    }

    #[sqlx::test]
    async fn test_create_log_with_null_admin_id(pool: PgPool) {
        let repo = AuditLogRepository::new(Arc::new(pool));
        let new_log = NewAuditLog {
            admin_user_id: None, // This is the key part of the test
            customer_id: Some(123),
            action: AuditAction::CustomerUpdate,
            status: AuditStatus::Success,
            target_table: "customers".to_string(),
            target_id: "123".to_string(),
            old_values: None,
            new_values: None,
            ip_address: None,
            user_agent: Some("bot".to_string()),
            request_id: None,
            error_message: None,
        };

        // This call will fail if the SELECT query after the INSERT has a column
        // ordering mismatch, causing the "unexpected null" error on a non-Option field.
        let result = repo.create(new_log).await.unwrap();

        // Verify that the LEFT JOIN produced a None for the login
        assert_eq!(result.admin_user_id, None);
        assert_eq!(result.admin_user_login, None);
        assert_eq!(result.customer_id, Some(123));
    }
}
