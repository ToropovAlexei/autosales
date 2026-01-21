use crate::AppState;
use crate::api::backend_api::BackendApi;
use crate::bot::{BotUsername, run_bot};
use crate::errors::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

pub struct BotManager {
    bots: HashMap<(String, i64), JoinHandle<()>>,
    app_state: AppState,
}

impl BotManager {
    pub fn new(app_state: AppState) -> Self {
        Self {
            bots: HashMap::new(),
            app_state,
        }
    }

    pub async fn start_bots(&mut self) -> anyhow::Result<()> {
        let all_bots = self.app_state.api.get_bots().await?;
        let fallback_bot = all_bots
            .items
            .iter()
            .find(|b| b.is_active && b.owner_id.is_none() && !b.is_primary)
            .map(|b| BotUsername(b.username.clone()));

        let bots_to_start = all_bots
            .items
            .iter()
            .filter(|b| b.is_primary)
            .collect::<Vec<_>>();

        for bot in bots_to_start {
            let fallback_bot_name = if bot.owner_id.is_some() {
                None
            } else {
                fallback_bot.clone()
            };
            self.start_bot(bot.token.clone(), bot.id, fallback_bot_name);
        }

        Ok(())
    }

    fn start_bot(
        &mut self,
        bot_token: String,
        bot_id: i64,
        fallback_bot_name: Option<BotUsername>,
    ) {
        let app_state = self.app_state.clone();
        let bot_token_clone = bot_token.clone();
        let handle = tokio::spawn(async move {
            loop {
                let app_state = app_state.clone();
                let token_clone = bot_token.clone();
                let api = Arc::new(
                    BackendApi::new(
                        &app_state.config.backend_api_url,
                        &app_state.config.service_api_key,
                        Some(bot_id),
                    )
                    .expect("Failed to create BackendApi"),
                );
                let fallback_bot_name = fallback_bot_name.clone();
                match run_bot(token_clone, bot_id, app_state, api, fallback_bot_name).await {
                    Ok(_) => break,
                    Err(e) => match e {
                        AppError::BotHealthcheckFailed(msg) => {
                            tracing::error!("Bot healthcheck failed: {}", msg);
                        }
                        AppError::BotUnauthorized(_) => {
                            // TODO Mark bot as inactive
                            break;
                        }
                        _ => {
                            tracing::error!("Error running bot: {}", e);
                        }
                    },
                }
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        });
        self.bots.insert((bot_token_clone, bot_id), handle);
    }

    pub async fn wait_for_all(&mut self) {
        for (_, handle) in self.bots.iter_mut() {
            handle.await.unwrap();
        }
    }
}
