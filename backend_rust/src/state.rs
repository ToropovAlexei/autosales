use std::sync::Arc;

use crate::{
    config::{self, Config},
    db,
};

#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
    pub config: config::Config,
}

impl AppState {
    pub fn new(db: db::Database, config: Config) -> Self {
        let db_pool = Arc::new(db.get_pool().clone());
        Self { db, config }
    }
}
