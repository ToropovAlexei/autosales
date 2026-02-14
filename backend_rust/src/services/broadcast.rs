use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use shared_dtos::audit_log::{AuditAction, AuditStatus};
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        audit_log::AuditLogRepository,
        broadcast::{BroadcastRepository, BroadcastRepositoryTrait},
    },
    middlewares::context::RequestContext,
    models::{
        audit_log::NewAuditLog,
        broadcast::{
            BroadcastListQuery, BroadcastRow, BroadcastStatus, NewBroadcast, UpdateBroadcast,
        },
        common::PaginatedResult,
    },
    presentation::admin::dtos::broadcast::JsonRawListQuery,
    services::audit_log::{AuditLogService, AuditLogServiceTrait},
};

#[derive(Debug)]
pub struct CreateBroadcastCommand {
    pub content_text: Option<String>,
    pub content_image_id: Option<Uuid>,
    pub filters: Option<JsonRawListQuery>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub created_by: i64,
    pub ctx: Option<RequestContext>,
}

#[derive(Debug)]
pub struct UpdateBroadcastCommand {
    pub id: i64,
    pub status: Option<BroadcastStatus>,
    pub content_text: Option<Option<String>>,
    pub content_image_id: Option<Option<Uuid>>,
    pub filters: Option<JsonRawListQuery>,
    pub scheduled_for: Option<Option<DateTime<Utc>>>,
    pub updated_by: Option<i64>,
    pub statistics: Option<Option<serde_json::Value>>,
    pub started_at: Option<Option<DateTime<Utc>>>,
    pub finished_at: Option<Option<DateTime<Utc>>>,
    pub ctx: Option<RequestContext>,
}

#[async_trait]
pub trait BroadcastServiceTrait: Send + Sync {
    async fn get_list(&self, query: BroadcastListQuery)
    -> ApiResult<PaginatedResult<BroadcastRow>>;
    async fn create(&self, command: CreateBroadcastCommand) -> ApiResult<BroadcastRow>;
    async fn update(&self, command: UpdateBroadcastCommand) -> ApiResult<BroadcastRow>;
    async fn get_ready_broadcasts(&self) -> ApiResult<Vec<BroadcastRow>>;
}

pub struct BroadcastService<R, A> {
    repo: Arc<R>,
    audit_log_service: Arc<A>,
}

impl<R, A> BroadcastService<R, A>
where
    R: BroadcastRepositoryTrait + Send + Sync,
    A: AuditLogServiceTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>, audit_log_service: Arc<A>) -> Self {
        Self {
            repo,
            audit_log_service,
        }
    }
}

#[async_trait]
impl BroadcastServiceTrait
    for BroadcastService<BroadcastRepository, AuditLogService<AuditLogRepository>>
{
    async fn get_list(
        &self,
        query: BroadcastListQuery,
    ) -> ApiResult<PaginatedResult<BroadcastRow>> {
        self.repo.get_list(query).await.map_err(ApiError::from)
    }

    async fn create(&self, command: CreateBroadcastCommand) -> ApiResult<BroadcastRow> {
        let created = self
            .repo
            .create(NewBroadcast {
                status: if command.scheduled_for.is_some() {
                    BroadcastStatus::Scheduled
                } else {
                    BroadcastStatus::Pending
                },
                content_text: command.content_text,
                content_image_id: command.content_image_id,
                filters: command
                    .filters
                    .map(|f| serde_json::to_value(f).unwrap_or_default()),
                created_by: command.created_by,
                scheduled_for: command.scheduled_for,
            })
            .await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::BroadcastCreate,
                status: AuditStatus::Success,
                admin_user_id: Some(command.created_by),
                customer_id: None,
                error_message: None,
                new_values: serde_json::to_value(created.clone()).ok(),
                old_values: None,
                target_id: created.id.to_string(),
                target_table: "broadcasts".to_string(),
                ip_address: command.ctx.clone().and_then(|ctx| ctx.ip_address),
                request_id: command.ctx.clone().map(|ctx| ctx.request_id),
                user_agent: command.ctx.and_then(|ctx| ctx.user_agent),
            })
            .await?;

        Ok(created)
    }

    async fn update(&self, command: UpdateBroadcastCommand) -> ApiResult<BroadcastRow> {
        let prev = self.repo.get_by_id(command.id).await?;
        let updated = self
            .repo
            .update(
                command.id,
                UpdateBroadcast {
                    content_image_id: command.content_image_id,
                    content_text: command.content_text,
                    filters: command
                        .filters
                        .map(|f| Some(serde_json::to_value(f).unwrap_or_default())),
                    scheduled_for: command.scheduled_for,
                    status: command.status,
                    started_at: command.started_at,
                    finished_at: command.finished_at,
                    statistics: command.statistics,
                },
            )
            .await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::BroadcastUpdate,
                status: AuditStatus::Success,
                admin_user_id: command.updated_by,
                customer_id: None,
                error_message: None,
                new_values: serde_json::to_value(updated.clone()).ok(),
                old_values: serde_json::to_value(prev.clone()).ok(),
                target_id: prev.id.to_string(),
                target_table: "broadcasts".to_string(),
                ip_address: command.ctx.clone().and_then(|ctx| ctx.ip_address),
                request_id: command.ctx.clone().map(|ctx| ctx.request_id),
                user_agent: command.ctx.and_then(|ctx| ctx.user_agent),
            })
            .await?;

        Ok(updated)
    }

    async fn get_ready_broadcasts(&self) -> ApiResult<Vec<BroadcastRow>> {
        self.repo
            .get_ready_broadcasts()
            .await
            .map_err(ApiError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::{
        audit_log::AuditLogRepository, broadcast::BroadcastRepository,
    };
    use crate::services::audit_log::AuditLogService;
    use chrono::Duration;
    use sqlx::PgPool;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::sync::Arc;

    fn build_service(
        pool: &PgPool,
    ) -> BroadcastService<BroadcastRepository, AuditLogService<AuditLogRepository>> {
        let pool = Arc::new(pool.clone());
        let audit_log_service = Arc::new(AuditLogService::new(Arc::new(AuditLogRepository::new(
            pool.clone(),
        ))));
        BroadcastService::new(Arc::new(BroadcastRepository::new(pool)), audit_log_service)
    }

    fn build_context() -> RequestContext {
        RequestContext {
            ip_address: Some(IpAddr::from_str("127.0.0.1").unwrap()),
            user_agent: Some("test-agent".to_string()),
            request_id: uuid::Uuid::new_v4(),
        }
    }

    async fn create_admin_user(pool: &PgPool, login: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO admin_users (login, hashed_password, two_fa_secret, created_by)
            VALUES ($1, 'password', '', 1)
            RETURNING id
            "#,
            login
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_update_broadcast(pool: PgPool) {
        let service = build_service(&pool);
        let admin_id = create_admin_user(&pool, "broadcast_admin_1").await;

        let created = service
            .create(CreateBroadcastCommand {
                content_text: Some("Hello".to_string()),
                content_image_id: None,
                filters: None,
                scheduled_for: None,
                created_by: admin_id,
                ctx: Some(build_context()),
            })
            .await
            .unwrap();

        assert_eq!(created.status, BroadcastStatus::Pending);

        let updated = service
            .update(UpdateBroadcastCommand {
                id: created.id,
                status: Some(BroadcastStatus::Completed),
                content_text: Some(Some("Updated".to_string())),
                content_image_id: None,
                filters: None,
                scheduled_for: None,
                updated_by: Some(admin_id),
                statistics: None,
                started_at: None,
                finished_at: None,
                ctx: Some(build_context()),
            })
            .await
            .unwrap();

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.status, BroadcastStatus::Completed);
        assert_eq!(updated.content_text, Some("Updated".to_string()));
    }

    #[sqlx::test]
    async fn test_get_ready_broadcasts(pool: PgPool) {
        let service = build_service(&pool);
        let admin_id = create_admin_user(&pool, "broadcast_admin_2").await;

        service
            .create(CreateBroadcastCommand {
                content_text: Some("Now".to_string()),
                content_image_id: None,
                filters: None,
                scheduled_for: None,
                created_by: admin_id,
                ctx: Some(build_context()),
            })
            .await
            .unwrap();

        service
            .create(CreateBroadcastCommand {
                content_text: Some("Later".to_string()),
                content_image_id: None,
                filters: None,
                scheduled_for: Some(Utc::now() + Duration::minutes(10)),
                created_by: admin_id,
                ctx: Some(build_context()),
            })
            .await
            .unwrap();

        let ready = service.get_ready_broadcasts().await.unwrap();
        assert!(
            ready
                .iter()
                .any(|b| b.content_text.as_deref() == Some("Now"))
        );
        assert!(
            !ready
                .iter()
                .any(|b| b.content_text.as_deref() == Some("Later"))
        );
    }
}
