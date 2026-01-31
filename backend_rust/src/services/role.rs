use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::api::{ApiError, ApiResult},
    infrastructure::repositories::role::{RoleRepository, RoleRepositoryTrait},
    models::role::{NewRole, RoleRow, UpdateRole},
};

#[async_trait]
pub trait RoleServiceTrait: Send + Sync {
    async fn get_list(&self) -> ApiResult<Vec<RoleRow>>;
    async fn create(&self, role: NewRole) -> ApiResult<RoleRow>;
    async fn update(&self, id: i64, role: UpdateRole) -> ApiResult<RoleRow>;
    async fn delete(&self, id: i64) -> ApiResult<()>;
}

pub struct RoleService<R> {
    repo: Arc<R>,
}

impl<R> RoleService<R>
where
    R: RoleRepositoryTrait + Send + Sync,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl RoleServiceTrait for RoleService<RoleRepository> {
    async fn get_list(&self) -> ApiResult<Vec<RoleRow>> {
        self.repo.get_roles().await.map_err(ApiError::from)
    }

    async fn create(&self, role: NewRole) -> ApiResult<RoleRow> {
        let created = self.repo.create_role(role).await?;

        Ok(created)
    }

    async fn update(&self, id: i64, role: UpdateRole) -> ApiResult<RoleRow> {
        let updated = self.repo.update_role(id, role).await?;

        Ok(updated)
    }

    async fn delete(&self, id: i64) -> ApiResult<()> {
        Ok(self.repo.delete_role(id).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::role::RoleRepository;
    use sqlx::PgPool;
    use std::sync::Arc;

    fn build_service(pool: &PgPool) -> RoleService<RoleRepository> {
        let pool = Arc::new(pool.clone());
        RoleService::new(Arc::new(RoleRepository::new(pool)))
    }

    #[sqlx::test]
    async fn test_create_update_delete_role(pool: PgPool) {
        let service = build_service(&pool);

        let created = service
            .create(NewRole {
                name: "role_test".to_string(),
                description: Some("desc".to_string()),
                created_by: 1,
            })
            .await
            .unwrap();

        assert_eq!(created.name, "role_test");

        let updated = service
            .update(
                created.id,
                UpdateRole {
                    name: Some("role_test_updated".to_string()),
                    description: Some(Some("new desc".to_string())),
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.name, "role_test_updated");

        service.delete(created.id).await.unwrap();
    }
}
