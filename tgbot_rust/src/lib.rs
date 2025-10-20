pub mod api;
pub mod bot;
pub mod config;
pub mod errors;
pub mod models;
pub mod monitor;
pub mod webhook;
pub mod utils;

use std::{io, sync::Arc};

use config::Config;
use deadpool_redis::{Pool, Runtime};

use tracing_appender::rolling;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, time::LocalTime},
    prelude::*,
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub redis_pool: Pool,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(config: Arc<Config>) -> Self {
        let redis_pool = create_redis_pool(&config);
        Self { config, redis_pool }
    }
}

pub fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let time_format = LocalTime::rfc_3339();

    let console_layer = fmt::layer()
        .with_timer(time_format.clone())
        .with_writer(io::stdout)
        .with_target(false)
        .with_level(true)
        .pretty();

    let file_appender = rolling::daily("logs", "app.log");
    let file_layer = fmt::layer()
        .with_timer(time_format)
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(false)
        .with_level(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();
}

pub fn create_redis_pool(config: &Config) -> Pool {
    let redis_url = format!("redis://{}:{}", config.redis_host, config.redis_port);

    let pool_cfg = deadpool_redis::Config::from_url(redis_url);

    pool_cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}
