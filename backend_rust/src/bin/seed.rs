use std::sync::Arc;

use backend_rust::{
    config::Config,
    db::Database,
    infrastructure::repositories::admin_user::{AdminUserRepository, AdminUserRepositoryTrait},
    init_tracing,
    models::admin_user::NewAdminUser,
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
    let admin_user_repo = Arc::new(AdminUserRepository::new(db_pool));
    create_dev_admin_if_not_exists(&admin_user_repo, &totp_encryptor).await;

    Ok(())
}

async fn create_dev_admin_if_not_exists(
    admin_user_repo: &Arc<AdminUserRepository>,
    totp_encryptor: &TotpEncryptor,
) {
    if admin_user_repo.get_by_login("admin").await.is_ok() {
        return;
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
        .unwrap();
}
