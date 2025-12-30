use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    errors::repository::RepositoryResult, models::admin_user_with_roles::AdminUserWithRolesRow,
};

#[async_trait]
pub trait AdminUserWithRolesRepositoryTrait {
    async fn get_list(&self) -> RepositoryResult<Vec<AdminUserWithRolesRow>>;
}

#[derive(Clone)]
pub struct AdminUserWithRolesRepository {
    pool: Arc<PgPool>,
}

impl AdminUserWithRolesRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdminUserWithRolesRepositoryTrait for AdminUserWithRolesRepository {
    async fn get_list(&self) -> RepositoryResult<Vec<AdminUserWithRolesRow>> {
        let result = sqlx::query_as(
            r#"SELECT
                    au.id,
                    au.login,
                    au.hashed_password,
                    au.two_fa_secret,
                    au.telegram_id,
                    au.is_system,
                    au.created_at,
                    au.updated_at,
                    au.deleted_at,
                    au.created_by,
                    COALESCE(
                        jsonb_agg(
                            json_build_object(
                                'id', r.id,
                                'name', r.name
                            ) ORDER BY r.name
                        ) FILTER (WHERE r.id IS NOT NULL),
                        '[]'::jsonb
                    ) AS roles
                FROM admin_users au
                LEFT JOIN user_roles ur ON ur.user_id = au.id
                LEFT JOIN roles r ON r.id = ur.role_id
                WHERE au.deleted_at IS NULL AND au.is_system = false
                GROUP BY
                    au.id,
                    au.login,
                    au.hashed_password,
                    au.two_fa_secret,
                    au.telegram_id,
                    au.is_system,
                    au.created_at,
                    au.updated_at,
                    au.deleted_at,
                    au.created_by"#,
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(result)
    }
}
