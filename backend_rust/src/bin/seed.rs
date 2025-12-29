use std::sync::Arc;

use backend_rust::{
    config::Config,
    db::Database,
    infrastructure::repositories::{
        admin_user::{AdminUserRepository, AdminUserRepositoryTrait},
        permission::{PermissionRepository, PermissionRepositoryTrait},
        role::{RoleRepository, RoleRepositoryTrait},
        role_permission::{RolePermissionRepository, RolePermissionRepositoryTrait},
        user_role::{UserRoleRepository, UserRoleRepositoryTrait},
    },
    init_tracing,
    models::{
        admin_user::NewAdminUser, role::NewRole, role_permission::NewRolePermission,
        user_role::NewUserRole,
    },
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
    let user_role_repo = Arc::new(UserRoleRepository::new(db_pool));
    let admin_id = create_dev_admin_if_not_exists(&admin_user_repo, &totp_encryptor).await;
    println!("Admin Id: {}", admin_id);
    let admin_role_id = create_dev_admin_role_if_not_exists(&role_repo).await;
    println!("Admin role Id: {}", admin_role_id);
    let assigned_permissions = assign_all_permissions_to_admin_role(
        admin_role_id,
        &permission_repo,
        &role_permission_repo,
    )
    .await;
    println!("Assigned permissions: {}", assigned_permissions);
    assign_admin_role_to_admin_user(admin_id, admin_role_id, &user_role_repo).await;
    println!("Admin user role assigned");

    Ok(())
}

async fn create_dev_admin_if_not_exists(
    admin_user_repo: &Arc<AdminUserRepository>,
    totp_encryptor: &TotpEncryptor,
) -> i64 {
    if let Ok(admin) = admin_user_repo.get_by_login("admin").await {
        return admin.id;
    }
    admin_user_repo
        .create(NewAdminUser {
            created_by: 1, // System
            hashed_password: bcrypt::hash("password", bcrypt::DEFAULT_COST).unwrap(),
            login: "admin".to_string(),
            telegram_id: None,
            two_fa_secret: totp_encryptor
                .encrypt("QO4C6IF3RRNNUXLKAIVLOQPVYM5W3XEV")
                .unwrap(),
        })
        .await
        .unwrap()
        .id
}

async fn create_dev_admin_role_if_not_exists(role_repo: &Arc<RoleRepository>) -> i64 {
    if let Ok(roles) = role_repo.get_roles().await
        && let Some(admin_role) = roles.iter().find(|role| role.name == "admin")
    {
        return admin_role.id;
    }
    role_repo
        .create_role(NewRole {
            name: "admin".to_string(),
            created_by: 1, // System,
            description: None,
        })
        .await
        .unwrap()
        .id
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

async fn assign_admin_role_to_admin_user(
    admin_user_id: i64,
    admin_role_id: i64,
    user_role_repo: &Arc<UserRoleRepository>,
) {
    if let Ok(user_roles) = user_role_repo.get_user_roles(admin_user_id).await
        && user_roles.is_empty()
    {
        user_role_repo
            .create_user_role(NewUserRole {
                user_id: admin_user_id,
                role_id: admin_role_id,
                created_by: 1, // System
            })
            .await
            .unwrap();
    }
}
