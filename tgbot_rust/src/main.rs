use std::sync::Arc;

use anyhow::Context;
use tgbot_rust::bot_manager::BotManager;
use tgbot_rust::config::Config;
use tgbot_rust::webhook::create_webhook_service;
use tgbot_rust::{AppState, create_redis_pool, init_logging};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    tracing::info!("Starting application");
    let config = Arc::new(Config::from_env());
    let redis_pool = create_redis_pool(&config).await;
    let app_state = AppState::new(config.clone(), redis_pool);
    let webhook_service = create_webhook_service(app_state.clone());
    let listener_address = format!("{}:{}", config.webhook_host, config.webhook_port);

    let listener = tokio::net::TcpListener::bind(&listener_address)
        .await
        .context("Failed to bind TCP listener")?;

    let bot_manager = Arc::new(tokio::sync::Mutex::new(BotManager::new(app_state)));
    bot_manager.lock().await.start_bots().await?;

    let server = axum::serve(listener, webhook_service);

    let mut bot_manager_guard = bot_manager.lock().await;
    tokio::select! {
        _ = bot_manager_guard.wait_for_all() => {
            tracing::info!("All bots have exited.");
        },
        _ = server => {},
    }

    Ok(())
}
