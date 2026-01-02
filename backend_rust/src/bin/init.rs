use std::sync::Arc;

use backend_rust::{
    bin::{
        assign_role_to_admin_user, create_admin_user_if_not_exists,
        create_admin_user_role_if_not_exists,
    },
    config::Config,
    db::Database,
    infrastructure::repositories::{
        admin_user::AdminUserRepository,
        permission::{PermissionRepository, PermissionRepositoryTrait},
        role::RoleRepository,
        role_permission::{RolePermissionRepository, RolePermissionRepositoryTrait},
        user_role::UserRoleRepository,
    },
    init_tracing,
    models::role_permission::NewRolePermission,
    run_migrations,
    services::topt_encryptor::TotpEncryptor,
    state::AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let config = Config::from_env();
    let pool = Database::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database_user,
        config.database_password,
        config.database_host,
        config.database_port,
        config.database_name,
    ))
    .await;
    run_migrations(&pool.pool).await?;
    let db_pool = Arc::new(pool.get_pool().clone());
    let _app_state = Arc::new(AppState::new(pool.clone(), config.clone()));
    let totp_encryptor = Arc::new(
        TotpEncryptor::new(&config.totp_encode_secret.clone())
            .expect("Failed to init totp_encryptor"),
    );
    let admin_user_repo = Arc::new(AdminUserRepository::new(db_pool.clone()));
    let role_repo = Arc::new(RoleRepository::new(db_pool.clone()));
    let permission_repo = Arc::new(PermissionRepository::new(db_pool.clone()));
    let role_permission_repo = Arc::new(RolePermissionRepository::new(db_pool.clone()));
    let user_role_repo = Arc::new(UserRoleRepository::new(db_pool.clone()));
    let admin_id = create_admin_user_if_not_exists(
        &admin_user_repo,
        &totp_encryptor,
        "admin".to_string(),
        None,
        None,
    )
    .await;
    println!("Admin Id: {}", admin_id);
    let admin_role_id = create_admin_user_role_if_not_exists(&role_repo, "admin").await;
    println!("Admin role Id: {}", admin_role_id);
    let assigned_permissions = assign_all_permissions_to_admin_role(
        admin_role_id,
        &permission_repo,
        &role_permission_repo,
    )
    .await;
    println!("Assigned permissions: {}", assigned_permissions);
    assign_role_to_admin_user(admin_id, admin_role_id, &user_role_repo).await;
    println!("Admin user role assigned");

    Ok(())
}

async fn assign_all_permissions_to_admin_role(
    role_id: i64,
    permission_repo: &Arc<PermissionRepository>,
    role_permission_repo: &Arc<RolePermissionRepository>,
) -> i64 {
    let all_permissions = permission_repo.get_list().await.unwrap();
    let mut required_permissions = Vec::<i64>::new();
    if let Ok(permissions) = role_permission_repo.get_role_permissions(role_id).await {
        all_permissions.iter().for_each(|p| {
            if !permissions.iter().any(|rp| rp.permission_id == p.id) {
                required_permissions.push(p.id);
            }
        });
    }
    if required_permissions.is_empty() {
        return 0;
    }
    let total = required_permissions.len() as i64;
    for permission in required_permissions {
        role_permission_repo
            .create_role_permission(NewRolePermission {
                role_id,
                permission_id: permission,
                created_by: 1, // System
            })
            .await
            .unwrap();
    }
    total
}
