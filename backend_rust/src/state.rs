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
        Self { db, config }
    }
}
