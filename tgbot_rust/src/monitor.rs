use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::{AppState, api::backend_api::BackendApi, bot::start_bot, errors::AppResult};

pub async fn manage_main_bots(app_state: AppState) -> AppResult<()> {
    let api_client = Arc::new(BackendApi::new(
        &app_state.config.backend_api_url,
        &app_state.config.service_token,
    )?);

    let bot_tokens = vec!["8322722853:AAG8g5_iLizghvlBg2MgBoKOcgRJ32mf9KM"];

    let cancel_token = CancellationToken::new();

    for token in bot_tokens {
        let app_state = app_state.clone();
        let api_client = api_client.clone();
        let cancel_token = cancel_token.clone();
        tokio::spawn(async move {
            if let Err(e) = start_bot(app_state, token, api_client, cancel_token).await {
                tracing::error!("Bot with token {token} failed: {e:?}");
            }
        });
    }

    tokio::signal::ctrl_c().await?;
    cancel_token.cancel();
    tracing::info!("Shutdown signal received, main manager exiting.");
    Ok(())
}
