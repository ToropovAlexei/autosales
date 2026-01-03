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
