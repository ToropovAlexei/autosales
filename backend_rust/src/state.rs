use std::sync::Arc;

use crate::{
    config::{self, Config},
    db,
    infrastructure::repositories::{
        active_token::ActiveTokenRepository, category::CategoryRepository,
    },
    services::{auth::AuthService, category::CategoryService},
};

#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
    pub config: config::Config,
    pub auth_service: Arc<AuthService<ActiveTokenRepository>>,
    pub category_service: Arc<CategoryService<CategoryRepository>>,
}

impl AppState {
    pub fn new(db: db::Database, config: Config) -> Self {
        let db_pool = Arc::new(db.get_pool().clone());
        let active_token_repo = Arc::new(ActiveTokenRepository::new(db_pool.clone()));
        let auth_service = Arc::new(AuthService::new(
            config.jwt_secret.clone(),
            active_token_repo,
        ));
        let category_repo = Arc::new(CategoryRepository::new(db_pool.clone()));
        let category_service = Arc::new(CategoryService::new(category_repo));

        Self {
            db,
            config,
            auth_service,
            category_service,
        }
    }
}
