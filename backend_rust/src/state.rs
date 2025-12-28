use std::sync::Arc;

use chrono::Duration;
use totp_rs::Algorithm;

use crate::{
    config::{self, Config},
    db,
    infrastructure::repositories::{
        active_token::ActiveTokenRepository, admin_user::AdminUserRepository,
        category::CategoryRepository, temporary_token::TemporaryTokenRepository,
    },
    services::{
        admin_user::AdminUserService,
        auth::{AuthService, AuthServiceConfig},
        category::CategoryService,
        topt_encryptor::TotpEncryptor,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
    pub config: config::Config,
    pub auth_service:
        Arc<AuthService<ActiveTokenRepository, TemporaryTokenRepository, AdminUserRepository>>,
    pub category_service: Arc<CategoryService<CategoryRepository>>,
    pub admin_user_service: Arc<AdminUserService<AdminUserRepository>>,
}

impl AppState {
    pub fn new(db: db::Database, config: Config) -> Self {
        let db_pool = Arc::new(db.get_pool().clone());
        let active_token_repo = Arc::new(ActiveTokenRepository::new(db_pool.clone()));
        let temp_token_repo = Arc::new(TemporaryTokenRepository::new(db_pool.clone()));
        let admin_user_repo = Arc::new(AdminUserRepository::new(db_pool.clone()));
        let totp_encryptor = Arc::new(
            TotpEncryptor::new(&config.totp_encode_secret.clone())
                .expect("Failed to init totp_encryptor"),
        );
        let auth_service = Arc::new(AuthService::new(
            active_token_repo,
            temp_token_repo,
            admin_user_repo.clone(),
            totp_encryptor.clone(),
            AuthServiceConfig {
                jwt_secret: config.jwt_secret.clone(),
                totp_encode_secret: config.totp_encode_secret.clone(),
                two_fa_token_ttl: Duration::minutes(config.two_fa_token_ttl_minutes),
                totp_algorithm: Algorithm::SHA1,
                totp_digits: 6,
                totp_skew: 1,
                totp_step: 30,
                access_token_ttl: Duration::minutes(config.access_token_ttl_minutes),
                refresh_token_ttl: Duration::minutes(config.refresh_token_ttl_minutes),
            },
        ));
        let category_repo = Arc::new(CategoryRepository::new(db_pool.clone()));
        let category_service = Arc::new(CategoryService::new(category_repo));
        let admin_user_service = Arc::new(AdminUserService::new(admin_user_repo, totp_encryptor));

        Self {
            db,
            config,
            auth_service,
            category_service,
            admin_user_service,
        }
    }
}
