use std::sync::Arc;

use crate::{
    config::{self, Config},
    db,
    infrastructure::repositories::active_token::ActiveTokenRepository,
    services::auth::AuthService,
};

#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
    pub config: config::Config,
    pub auth_service: Arc<AuthService<ActiveTokenRepository>>,
}

impl AppState {
    pub fn new(db: db::Database, config: Config) -> Self {
        let db_pool = Arc::new(db.get_pool().clone());
        let active_token_repo = Arc::new(ActiveTokenRepository::new(db_pool.clone()));
        let auth_service = Arc::new(AuthService::new(
            config.jwt_secret.clone(),
            active_token_repo,
        ));

        Self {
            db,
            config,
            auth_service,
        }
    }
}
