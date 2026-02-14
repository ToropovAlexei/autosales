use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use shared_dtos::audit_log::{AuditAction, AuditStatus};
use uuid::Uuid;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::{
        audit_log::AuditLogRepository,
        category::{CategoryRepository, CategoryRepositoryTrait},
    },
    middlewares::context::RequestContext,
    models::{
        audit_log::NewAuditLog,
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
    pub ctx: Option<RequestContext>,
}

#[derive(Debug)]
pub struct CreateCategorySequenceCommand {
    pub name: String,
    pub created_by: i64,
    pub ctx: Option<RequestContext>,
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
    async fn create(&self, command: CreateCategoryCommand) -> ApiResult<CategoryRow>;
    async fn get_by_id(&self, id: i64) -> ApiResult<CategoryRow>;
    async fn update(
        &self,
        command: UpdateCategoryCommand,
        ctx: RequestContext,
    ) -> ApiResult<CategoryRow>;
    async fn delete(&self, command: DeleteCategoryCommand, ctx: RequestContext) -> ApiResult<()>;
    async fn create_category_sequence(
        &self,
        command: CreateCategorySequenceCommand,
    ) -> ApiResult<Option<CategoryRow>>;
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

    async fn create(&self, command: CreateCategoryCommand) -> ApiResult<CategoryRow> {
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
                new_values: serde_json::to_value(created.clone()).ok(),
                old_values: None,
                target_id: created.id.to_string(),
                target_table: "categories".to_string(),
                ip_address: command.ctx.clone().and_then(|ctx| ctx.ip_address),
                request_id: command.ctx.clone().map(|ctx| ctx.request_id),
                user_agent: command.ctx.and_then(|ctx| ctx.user_agent),
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

    async fn create_category_sequence(
        &self,
        command: CreateCategorySequenceCommand,
    ) -> ApiResult<Option<CategoryRow>> {
        let categories_sequence = command.name.split('/').filter(|s| !s.is_empty());
        let mut categories_by_name_and_parent = self.get_list().await.map(|rows| {
            rows.into_iter()
                .map(|row| ((row.name.clone(), row.parent_id), row))
                .collect::<HashMap<_, _>>()
        })?;

        let mut parent_id = None;
        let mut last_category = None;
        for category in categories_sequence {
            if let Some(row) = categories_by_name_and_parent.get(&(category.to_string(), parent_id))
            {
                parent_id = Some(row.id);
                last_category = Some(row.clone());
                continue;
            }

            let created = self
                .create(CreateCategoryCommand {
                    name: category.to_string(),
                    parent_id,
                    image_id: None,
                    created_by: command.created_by,
                    ctx: command.ctx.clone(),
                })
                .await?;
            categories_by_name_and_parent
                .insert((category.to_string(), parent_id), created.clone());
            parent_id = Some(created.id);
            last_category = Some(created);
        }
        Ok(last_category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::{
        audit_log::AuditLogRepository, category::CategoryRepository,
    };
    use crate::services::audit_log::AuditLogService;
    use sqlx::PgPool;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::sync::Arc;
    use uuid::Uuid;

    fn build_service(
        pool: &PgPool,
    ) -> CategoryService<CategoryRepository, AuditLogService<AuditLogRepository>> {
        let pool = Arc::new(pool.clone());
        let audit_log_service = Arc::new(AuditLogService::new(Arc::new(AuditLogRepository::new(
            pool.clone(),
        ))));
        CategoryService::new(Arc::new(CategoryRepository::new(pool)), audit_log_service)
    }

    fn build_context() -> RequestContext {
        RequestContext {
            ip_address: Some(IpAddr::from_str("127.0.0.1").unwrap()),
            user_agent: Some("test-agent".to_string()),
            request_id: Uuid::new_v4(),
        }
    }

    #[sqlx::test]
    async fn test_create_with_missing_parent(pool: PgPool) {
        let service = build_service(&pool);

        let err = service
            .create(CreateCategoryCommand {
                name: "Child".to_string(),
                parent_id: Some(999999),
                image_id: None,
                created_by: 1,
                ctx: Some(build_context()),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, ApiError::BadRequest(_)));
    }

    #[sqlx::test]
    async fn test_create_and_update(pool: PgPool) {
        let service = build_service(&pool);

        let created = service
            .create(CreateCategoryCommand {
                name: "Root".to_string(),
                parent_id: None,
                image_id: None,
                created_by: 1,
                ctx: Some(build_context()),
            })
            .await
            .unwrap();

        let updated = service
            .update(
                UpdateCategoryCommand {
                    id: created.id,
                    name: Some("Root Updated".to_string()),
                    parent_id: None,
                    image_id: None,
                    position: Some(5),
                    updated_by: 1,
                },
                build_context(),
            )
            .await
            .unwrap();

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "Root Updated");
        assert_eq!(updated.position, 5);
    }

    #[sqlx::test]
    async fn test_delete_rejects_parent_with_children(pool: PgPool) {
        let service = build_service(&pool);
        let parent = service
            .create(CreateCategoryCommand {
                name: "Parent".to_string(),
                parent_id: None,
                image_id: None,
                created_by: 1,
                ctx: Some(build_context()),
            })
            .await
            .unwrap();

        service
            .create(CreateCategoryCommand {
                name: "Child".to_string(),
                parent_id: Some(parent.id),
                image_id: None,
                created_by: 1,
                ctx: Some(build_context()),
            })
            .await
            .unwrap();

        let err = service
            .delete(
                DeleteCategoryCommand {
                    id: parent.id,
                    deleted_by: 1,
                },
                build_context(),
            )
            .await
            .unwrap_err();

        assert!(matches!(err, ApiError::BadRequest(_)));
    }

    #[sqlx::test]
    async fn test_create_category_sequence(pool: PgPool) {
        let service = build_service(&pool);
        let created = service
            .create_category_sequence(CreateCategorySequenceCommand {
                name: "Root/Sub".to_string(),
                created_by: 1,
                ctx: Some(build_context()),
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(created.name, "Sub");

        let root = sqlx::query_scalar!("SELECT id FROM categories WHERE name = 'Root'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(created.parent_id, Some(root));
    }
}
