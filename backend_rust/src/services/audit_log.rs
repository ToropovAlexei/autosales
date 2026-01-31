use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::audit_log::{AuditLogRepository, AuditLogRepositoryTrait},
    models::{
        audit_log::{AuditLogListQuery, AuditLogRow, NewAuditLog},
        common::PaginatedResult,
    },
};

#[async_trait]
pub trait AuditLogServiceTrait: Send + Sync {
    async fn get_list(&self, query: AuditLogListQuery) -> ApiResult<PaginatedResult<AuditLogRow>>;
    async fn create(&self, audit_log: NewAuditLog) -> ApiResult<AuditLogRow>;
}

pub struct AuditLogService<R> {
    audit_log_repo: Arc<R>,
}

impl<R> AuditLogService<R>
where
    R: AuditLogRepositoryTrait + Send + Sync,
{
    pub fn new(audit_log_repo: Arc<R>) -> Self {
        Self { audit_log_repo }
    }
}

#[async_trait]
impl AuditLogServiceTrait for AuditLogService<AuditLogRepository> {
    async fn get_list(&self, query: AuditLogListQuery) -> ApiResult<PaginatedResult<AuditLogRow>> {
        self.audit_log_repo
            .get_list(query)
            .await
            .map_err(ApiError::from)
    }

    async fn create(&self, customer: NewAuditLog) -> ApiResult<AuditLogRow> {
        let created = self.audit_log_repo.create(customer).await?;

        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::audit_log::AuditLogRepository;
    use crate::models::audit_log::{AuditAction, AuditStatus};
    use sqlx::PgPool;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::sync::Arc;
    use uuid::Uuid;

    fn build_service(pool: &PgPool) -> AuditLogService<AuditLogRepository> {
        let pool = Arc::new(pool.clone());
        AuditLogService::new(Arc::new(AuditLogRepository::new(pool)))
    }

    #[sqlx::test]
    async fn test_create_and_list(pool: PgPool) {
        let service = build_service(&pool);

        let created = service
            .create(NewAuditLog {
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
            })
            .await
            .unwrap();

        let list = service
            .get_list(AuditLogListQuery::default())
            .await
            .unwrap();
        assert!(list.total >= 1);
        assert!(list.items.iter().any(|item| item.id == created.id));
    }
}
