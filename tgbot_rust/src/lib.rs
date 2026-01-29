pub mod api;
pub mod bot;
pub mod bot_father;
pub mod bot_manager;
pub mod config;
pub mod errors;
pub mod webhook;

use std::{io, sync::Arc};

use config::Config;
use deadpool_redis::{Pool, Runtime, redis::AsyncCommands};

use tracing_appender::rolling;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, time::LocalTime},
    prelude::*,
};

use crate::api::backend_api::BackendApi;

#[derive(Clone)]
pub struct AppState {
    pub redis_pool: Pool,
    pub config: Arc<Config>,
    pub api: Arc<BackendApi>,
}

impl AppState {
    pub fn new(config: Arc<Config>, redis_pool: Pool) -> Self {
        let api = Arc::new(
            BackendApi::new(&config.backend_api_url, &config.service_api_key, None).unwrap(),
        );
        Self {
            config,
            redis_pool,
            api,
        }
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

pub async fn create_redis_pool(config: &Config) -> Pool {
    let redis_url = format!("redis://{}:{}", config.redis_host, config.redis_port);

    let pool_cfg = deadpool_redis::Config::from_url(redis_url);

    let pool = pool_cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool config");

    let mut conn = pool.get().await.expect("Failed to get redis connection");
    let pong: String = conn.ping().await.expect("Failed to ping redis");

    if pong != "PONG" {
        panic!("Redis is not running");
    }

    pool
}
