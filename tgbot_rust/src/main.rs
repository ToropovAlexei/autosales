use std::sync::Arc;

use anyhow::Context;
use tgbot_rust::config::Config;
use tgbot_rust::monitor::manage_main_bots;
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

    let monitor_handle = tokio::spawn(manage_main_bots(app_state));

    let server = axum::serve(listener, webhook_service);

    tokio::select! {
        res = monitor_handle => {
            match res {
                Ok(Ok(())) => tracing::info!("Bot manager exited cleanly."),
                Ok(Err(e)) => tracing::error!("Bot manager exited with error: {e}"),
                Err(e) => tracing::error!("Bot manager task panicked: {e}"),
            }
        },
        res = server => {
            res.context("Server error")?;
        }
    }

    Ok(())
}
