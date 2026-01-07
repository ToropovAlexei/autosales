use sqlx::{Executor, Postgres};

use crate::{
    errors::repository::RepositoryResult, models::admin_user_with_roles::AdminUserWithRolesRow,
};

pub async fn get_admin_user_with_roles_list<'e, E>(
    executor: E,
) -> RepositoryResult<Vec<AdminUserWithRolesRow>>
where
    E: Executor<'e, Database = Postgres> + Send + Sync,
{
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
    .fetch_all(executor)
    .await?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn setup_test_data(executor: &PgPool) -> (i64, i64) {
        let mut tx = executor.begin().await.unwrap();

        let role_id = sqlx::query!(
            r#"INSERT INTO roles (name, created_by) VALUES ($1, $2) RETURNING id"#,
            "admin",
            1
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap()
        .id;

        let user_id = sqlx::query!(
            r#"INSERT INTO admin_users 
                (login, hashed_password, two_fa_secret, is_system, created_by) 
                VALUES ($1, $2, $3, $4, $5) 
                RETURNING id"#,
            "testuser",
            "hash",
            "secret",
            false,
            1i64
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap()
        .id;

        sqlx::query!(
            r#"INSERT INTO user_roles (user_id, role_id, created_by) VALUES ($1, $2, $3)"#,
            user_id,
            role_id,
            1
        )
        .execute(&mut *tx)
        .await
        .unwrap();

        let user_no_roles_id = sqlx::query!(
            r#"INSERT INTO admin_users 
                (login, hashed_password, two_fa_secret, is_system, created_by) 
                VALUES ($1, $2, $3, $4, $5) 
                RETURNING id"#,
            "noroles",
            "hash2",
            "secret2",
            false,
            1i64
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap()
        .id;

        tx.commit().await.unwrap();
        (user_id, user_no_roles_id)
    }

    #[sqlx::test]
    async fn test_get_list_returns_users_with_roles(executor: PgPool) -> Result<(), sqlx::Error> {
        let (user_id, user_no_roles_id) = setup_test_data(&executor).await;

        let rows = get_admin_user_with_roles_list(&executor).await.unwrap();

        assert_eq!(rows.len(), 2);

        let user_with_roles = rows.iter().find(|u| u.login == "testuser").unwrap();
        let user_no_roles = rows.iter().find(|u| u.login == "noroles").unwrap();

        assert_eq!(user_with_roles.roles.len(), 1);
        assert_eq!(user_with_roles.roles[0].name, "admin");
        assert_eq!(user_with_roles.id, user_id);

        assert_eq!(user_no_roles.roles.len(), 0);
        assert_eq!(user_no_roles.id, user_no_roles_id);

        assert!(!user_with_roles.is_system);
        assert!(user_with_roles.hashed_password.contains("hash"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_list_excludes_deleted_and_system_users(
        executor: PgPool,
    ) -> Result<(), sqlx::Error> {
        let mut tx = executor.begin().await?;

        sqlx::query!(
            r#"INSERT INTO admin_users 
                (login, hashed_password, two_fa_secret, is_system, created_by, deleted_at) 
                VALUES ($1, $2, $3, $4, $5, NOW())"#,
            "deleted_user",
            "hash",
            "secret",
            false,
            1i64
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"INSERT INTO admin_users 
                (login, hashed_password, two_fa_secret, is_system, created_by) 
                VALUES ($1, $2, $3, $4, $5)"#,
            "system_user",
            "",
            "",
            true,
            1i64
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let rows = get_admin_user_with_roles_list(&executor).await.unwrap();

        let logins: Vec<_> = rows.iter().map(|u| u.login.clone()).collect();
        assert!(!logins.contains(&"deleted_user".to_string()));
        assert!(!logins.contains(&"system_user".to_string()));

        Ok(())
    }
}
