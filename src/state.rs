use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    config::Config,
    db::Database,
    repositories::category::{CategoryRepository, CategoryRepositoryTrait},
    services::category::CategoryService,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Config,
    // Services
    pub category_service: Arc<CategoryService>,
}

impl AppState {
    pub fn new(db: Database, config: Config) -> Self {
        let pool = Arc::new(db.pool.clone());

        // Instantiate Repositories
        let category_repo: Arc<dyn CategoryRepositoryTrait> =
            Arc::new(CategoryRepository::new(pool.clone()));

        // Instantiate Services
        let category_service = Arc::new(CategoryService::new(category_repo));

        Self {
            db,
            config,
            category_service,
        }
    }
}
