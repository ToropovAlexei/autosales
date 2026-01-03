use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        audit_log::AuditLogRepository,
        category::{CategoryRepository, CategoryRepositoryTrait},
    },
    middlewares::context::RequestContext,
    models::{
        audit_log::{AuditAction, AuditStatus, NewAuditLog},
        category::{CategoryRow, NewCategory, UpdateCategory},
    },
    services::audit_log::{AuditLogService, AuditLogServiceTrait},
};

#[derive(Debug)]
pub struct CreateCategoryCommand {
    pub name: String,
    pub parent_id: Option<i64>,
    pub image_id: Option<Uuid>,
    pub created_by: i64,
}

#[derive(Debug)]
pub struct UpdateCategoryCommand {
    pub id: i64,
    pub name: Option<String>,
    pub parent_id: Option<Option<i64>>,
    pub image_id: Option<Option<Uuid>>,
    pub position: Option<i16>,
    pub updated_by: i64,
}

#[derive(Debug)]
pub struct DeleteCategoryCommand {
    pub id: i64,
    pub deleted_by: i64,
}

#[async_trait]
pub trait CategoryServiceTrait: Send + Sync {
    async fn get_list(&self) -> ApiResult<Vec<CategoryRow>>;
    async fn create(
        &self,
        command: CreateCategoryCommand,
        ctx: RequestContext,
    ) -> ApiResult<CategoryRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<CategoryRow>;
    async fn update(
        &self,
        command: UpdateCategoryCommand,
        ctx: RequestContext,
    ) -> ApiResult<CategoryRow>;
    async fn delete(&self, command: DeleteCategoryCommand, ctx: RequestContext) -> ApiResult<()>;
}

pub struct CategoryService<R, A> {
    repo: Arc<R>,
    audit_log_service: Arc<A>,
}

impl<R, A> CategoryService<R, A>
where
    R: CategoryRepositoryTrait + Send + Sync,
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
impl CategoryServiceTrait
    for CategoryService<CategoryRepository, AuditLogService<AuditLogRepository>>
{
    async fn get_list(&self) -> ApiResult<Vec<CategoryRow>> {
        self.repo.get_list().await.map_err(ApiError::from)
    }

    async fn create(
        &self,
        command: CreateCategoryCommand,
        ctx: RequestContext,
    ) -> ApiResult<CategoryRow> {
        if let Some(parent_id) = command.parent_id {
            let parent = self.repo.get_by_id(parent_id).await;

            if parent.is_err() {
                return Err(ApiError::BadRequest(
                    "Parent category does not exist".to_string(),
                ));
            }
        };

        let created = self
            .repo
            .create(NewCategory {
                name: command.name,
                parent_id: command.parent_id,
                image_id: command.image_id,
                created_by: command.created_by,
            })
            .await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::CategoryCreate,
                status: AuditStatus::Success,
                admin_user_id: Some(command.created_by),
                customer_id: None,
                error_message: None,
                ip_address: ctx.ip_address,
                new_values: serde_json::to_value(created.clone()).ok(),
                old_values: None,
                request_id: Some(ctx.request_id),
                target_id: created.id.to_string(),
                target_table: "categories".to_string(),
                user_agent: ctx.user_agent.clone(),
            })
            .await?;

        Ok(created)
    }

    async fn get_by_id(&self, id: i64) -> ApiResult<CategoryRow> {
        let res = self.repo.get_by_id(id).await?;
        Ok(res)
    }

    async fn update(
        &self,
        command: UpdateCategoryCommand,
        ctx: RequestContext,
    ) -> ApiResult<CategoryRow> {
        if let Some(ref name) = command.name
            && name.trim().is_empty()
        {
            return Err(ApiError::BadRequest(
                "Category name cannot be empty or whitespace only".to_string(),
            ));
        }

        if let Some(Some(new_parent_id)) = command.parent_id {
            if new_parent_id == command.id {
                return Err(ApiError::BadRequest(
                    "Cannot set parent to self".to_string(),
                ));
            }
            let parent = self.repo.get_by_id(new_parent_id).await;

            if parent.is_err() {
                return Err(ApiError::BadRequest(
                    "Parent category does not exist".to_string(),
                ));
            }
        }

        let prev = self.repo.get_by_id(command.id).await?;
        let updated = self
            .repo
            .update(
                command.id,
                UpdateCategory {
                    name: command.name,
                    parent_id: command.parent_id,
                    image_id: command.image_id,
                    position: command.position,
                },
            )
            .await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::CategoryUpdate,
                status: AuditStatus::Success,
                admin_user_id: Some(command.updated_by),
                customer_id: None,
                error_message: None,
                ip_address: ctx.ip_address,
                new_values: serde_json::to_value(updated.clone()).ok(),
                old_values: serde_json::to_value(prev.clone()).ok(),
                request_id: Some(ctx.request_id),
                target_id: prev.id.to_string(),
                target_table: "categories".to_string(),
                user_agent: ctx.user_agent.clone(),
            })
            .await?;

        Ok(updated)
    }

    async fn delete(&self, command: DeleteCategoryCommand, ctx: RequestContext) -> ApiResult<()> {
        if !self.repo.get_by_parent_id(command.id).await?.is_empty() {
            return Err(ApiError::BadRequest(
                "Cannot delete category with child categories".to_string(),
            ));
        }

        let prev = self.repo.get_by_id(command.id).await?;
        self.repo.delete(command.id).await?;

        self.audit_log_service
            .create(NewAuditLog {
                action: AuditAction::CategoryDelete,
                status: AuditStatus::Success,
                admin_user_id: Some(command.deleted_by),
                customer_id: None,
                error_message: None,
                ip_address: ctx.ip_address,
                new_values: None,
                old_values: serde_json::to_value(prev.clone()).ok(),
                request_id: Some(ctx.request_id),
                target_id: prev.id.to_string(),
                target_table: "categories".to_string(),
                user_agent: ctx.user_agent.clone(),
            })
            .await?;

        Ok(())
    }
}
