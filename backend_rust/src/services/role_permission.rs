use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::role_permission::{
        RolePermissionRepository, RolePermissionRepositoryTrait,
    },
    models::role_permission::{NewRolePermission, RolePermissionRow, UpdateRolePermissions},
};

#[async_trait]
pub trait RolePermissionServiceTrait: Send + Sync {
    async fn get_for_role(&self, role_id: i64) -> ApiResult<Vec<RolePermissionRow>>;
    async fn create(&self, permission: NewRolePermission) -> ApiResult<RolePermissionRow>;
    async fn delete(&self, role_id: i64, permission_id: i64) -> ApiResult<()>;
    async fn update_role_permissions(&self, permissions: UpdateRolePermissions) -> ApiResult<()>;
}

pub struct RolePermissionService<R> {
    repo: Arc<R>,
}

impl<R> RolePermissionService<R>
where
    R: RolePermissionRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl RolePermissionServiceTrait for RolePermissionService<RolePermissionRepository> {
    async fn get_for_role(&self, role_id: i64) -> ApiResult<Vec<RolePermissionRow>> {
        self.repo
            .get_role_permissions(role_id)
            .await
            .map_err(ApiError::from)
    }

    async fn create(&self, permission: NewRolePermission) -> ApiResult<RolePermissionRow> {
        let created = self.repo.create_role_permission(permission).await?;

        Ok(created)
    }

    async fn delete(&self, role_id: i64, permission_id: i64) -> ApiResult<()> {
        Ok(self
            .repo
            .delete_role_permission(role_id, permission_id)
            .await?)
    }

    async fn update_role_permissions(&self, permissions: UpdateRolePermissions) -> ApiResult<()> {
        Ok(self.repo.update_role_permissions(permissions).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::role_permission::RolePermissionRepository;
    use sqlx::PgPool;
    use std::sync::Arc;

    async fn create_role(pool: &PgPool, name: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO roles (name, description, created_by)
            VALUES ($1, NULL, 1)
            RETURNING id
            "#,
            name
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn create_permission(pool: &PgPool, name: &str, group: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO permissions ("group", name, description)
            VALUES ($1, $2, 'desc')
            RETURNING id
            "#,
            group,
            name
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    fn build_service(pool: &PgPool) -> RolePermissionService<RolePermissionRepository> {
        let pool = Arc::new(pool.clone());
        RolePermissionService::new(Arc::new(RolePermissionRepository::new(pool)))
    }

    #[sqlx::test]
    async fn test_create_and_delete(pool: PgPool) {
        let service = build_service(&pool);
        let role_id = create_role(&pool, "rp_role").await;
        let permission_id = create_permission(&pool, "rp:perm", "rp").await;

        let created = service
            .create(NewRolePermission {
                role_id,
                permission_id,
                created_by: 1,
            })
            .await
            .unwrap();

        assert_eq!(created.role_id, role_id);
        assert_eq!(created.permission_id, permission_id);

        service.delete(role_id, permission_id).await.unwrap();
    }

    #[sqlx::test]
    async fn test_update_role_permissions(pool: PgPool) {
        let service = build_service(&pool);
        let role_id = create_role(&pool, "rp_role_2").await;
        let permission_id = create_permission(&pool, "rp:perm2", "rp").await;

        service
            .update_role_permissions(UpdateRolePermissions {
                role_id,
                added: vec![permission_id],
                removed: vec![],
                created_by: 1,
            })
            .await
            .unwrap();

        let existing = service.get_for_role(role_id).await.unwrap();
        assert!(existing.iter().any(|p| p.permission_id == permission_id));
    }
}
