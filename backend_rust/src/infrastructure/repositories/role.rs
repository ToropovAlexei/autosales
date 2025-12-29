use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder};

use crate::{
    errors::repository::RepositoryResult,
    models::role::{NewRole, RoleRow, UpdateRole},
};

#[async_trait]
pub trait RoleRepositoryTrait {
    async fn get_roles(&self) -> RepositoryResult<Vec<RoleRow>>;
    async fn create_role(&self, role: NewRole) -> RepositoryResult<RoleRow>;
    async fn update_role(&self, role_id: i64, role: UpdateRole) -> RepositoryResult<RoleRow>;
    async fn delete_role(&self, role_id: i64) -> RepositoryResult<()>;
}

#[derive(Clone)]
pub struct RoleRepository {
    pool: Arc<PgPool>,
}

impl RoleRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepositoryTrait for RoleRepository {
    async fn get_roles(&self) -> RepositoryResult<Vec<RoleRow>> {
        let result = sqlx::query_as!(RoleRow, r#"SELECT * FROM roles"#,)
            .fetch_all(&*self.pool)
            .await?;
        Ok(result)
    }

    async fn create_role(&self, role: NewRole) -> RepositoryResult<RoleRow> {
        let result = sqlx::query_as!(
            RoleRow,
            r#"INSERT INTO roles (name, description, created_by)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            role.name,
            role.description,
            role.created_by,
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(result)
    }

    async fn update_role(&self, role_id: i64, role: UpdateRole) -> RepositoryResult<RoleRow> {
        let mut query_builder = QueryBuilder::new("UPDATE roles SET name = COALESCE($1, name)");

        if let Some(description) = role.description {
            query_builder.push(", description = ");
            query_builder.push_bind(description);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(role_id);
        query_builder.push(" RETURNING *");

        let query = query_builder.build_query_as::<RoleRow>();
        let result = query.fetch_one(&*self.pool).await?;
        Ok(result)
    }

    async fn delete_role(&self, role_id: i64) -> RepositoryResult<()> {
        sqlx::query!("DELETE FROM roles WHERE id = $1", role_id)
            .execute(&*self.pool)
            .await?;
        Ok(())
    }
}
