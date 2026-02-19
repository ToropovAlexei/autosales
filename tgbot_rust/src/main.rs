use std::sync::Arc;

use anyhow::Context;
use tgbot_rust::bot_manager::BotManager;
use tgbot_rust::config::Config;
use tgbot_rust::manager_bot::spawn_manager_bot_supervisor;
use tgbot_rust::webhook::create_webhook_service;
use tgbot_rust::{AppState, create_redis_pool, init_logging};
use tokio::signal;

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

    let bot_manager = Arc::new(BotManager::new(Arc::new(app_state.clone())));

    let server =
        axum::serve(listener, webhook_service).with_graceful_shutdown(shutdown_signal(app_state));

    tokio::spawn({
        let bot_manager = bot_manager.clone();
        async move { bot_manager.run().await }
    });

    spawn_manager_bot_supervisor(config.clone());

    if let Err(e) = server.await {
        tracing::error!(error = %e, "Axum server error");
    }

    Ok(())
}

async fn shutdown_signal(_state: AppState) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("received CTRL+C, shutting down...");
        }
        _ = terminate => {
            tracing::info!("received terminate signal, shutting down...");
        }
    };
}
