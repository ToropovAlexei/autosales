use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{errors::repository::RepositoryResult, models::permission::PermissionRow};

#[async_trait]
pub trait PermissionRepositoryTrait {
    async fn get_list(&self) -> RepositoryResult<Vec<PermissionRow>>;
}

#[derive(Clone)]
pub struct PermissionRepository {
    pool: Arc<PgPool>,
}

impl PermissionRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PermissionRepositoryTrait for PermissionRepository {
    async fn get_list(&self) -> RepositoryResult<Vec<PermissionRow>> {
        let result = sqlx::query_as!(PermissionRow, "SELECT * FROM permissions")
            .fetch_all(&*self.pool)
            .await?;
        Ok(result)
    }
}
