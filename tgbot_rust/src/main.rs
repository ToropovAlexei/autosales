use std::sync::Arc;

use anyhow::Context;
use tgbot_rust::bot_manager::BotManager;
use tgbot_rust::config::Config;
use tgbot_rust::health_checker::HealthChecker;
use tgbot_rust::webhook::create_webhook_service;
use tgbot_rust::{AppState, init_logging};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    tracing::info!("Starting application");
    let config = Arc::new(Config::from_env());
    let app_state = AppState::new(config.clone());
    let webhook_service = create_webhook_service(app_state.clone());
    let listener_address = format!("{}:{}", config.webhook_host, config.webhook_port);

    let listener = tokio::net::TcpListener::bind(&listener_address)
        .await
        .context("Failed to bind TCP listener")?;

    let bot_manager = Arc::new(tokio::sync::Mutex::new(BotManager::new(app_state)));
    bot_manager.lock().await.start_bots().await?;

    let health_checker = HealthChecker::new(bot_manager.clone());
    let health_checker_handle = tokio::spawn(async move {
        health_checker.start().await;
    });

    let server = axum::serve(listener, webhook_service);

    let mut bot_manager_guard = bot_manager.lock().await;
    tokio::select! {
        _ = bot_manager_guard.wait_for_all() => {
            tracing::info!("All bots have exited.");
        },
        _ = server => {},
        _ = health_checker_handle => {},
    }

    Ok(())
}
