use crate::AppState;
use crate::api::backend_api::BackendApi;
use crate::bot::run_bot;
use std::collections::HashMap;
use std::sync::Arc;
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

        let main_bots: Vec<_> = all_bots.items.iter().filter(|b| b.is_primary).collect();
        // let referral_bots: Vec<_> = all_bots.items.iter().filter(|b| !b.is_primary).collect();

        if let Some(main_bot) = main_bots.into_iter().find(|b| b.is_active) {
            self.start_bot(main_bot.token.clone(), main_bot.id);
        } else {
            tracing::warn!("No active main bots found, requesting a new one...");

            // let bot_father = BotFather::new(
            //     self.app_state.api.clone(),
            //     &self.app_state.config.telegram_api_id,
            //     &self.app_state.config.telegram_api_hash,
            // )?;

            // match bot_father.request_new_main_bot_token().await {
            //     Ok(true) => {
            //         tracing::info!(
            //             "Successfully requested and created a new bot. The bot manager will pick it up in the next health check cycle."
            //         );
            //     }
            //     Ok(false) => {
            //         tracing::error!("BotFather interaction finished, but no bot was created.");
            //     }
            //     Err(e) => {
            //         tracing::error!("Failed to request a new bot via BotFather: {}", e);
            //     }
            // }
        }

        // for referral_bot in referral_bots {
        //     if referral_bot.is_active {
        //         self.start_bot(referral_bot.token.clone(), referral_bot.id);
        //     }
        // }

        Ok(())
    }

    fn start_bot(&mut self, bot_token: String, bot_id: i64) {
        let app_state = self.app_state.clone();
        let token_clone = bot_token.clone();
        let handle = tokio::spawn(async move {
            let api = Arc::new(
                BackendApi::new(
                    &app_state.config.backend_api_url,
                    &app_state.config.service_api_key,
                    Some(bot_id),
                )
                .expect("Failed to create BackendApi"),
            );
            if let Err(e) = run_bot(token_clone, app_state, api).await {
                tracing::error!("Bot exited with error: {}", e);
            }
        });
        self.bots.insert((bot_token, bot_id), handle);
    }

    pub async fn wait_for_all(&mut self) {
        for (_, handle) in self.bots.iter_mut() {
            handle.await.unwrap();
        }
    }

    pub async fn check_bots_health(&mut self) {
        let mut bots_to_restart = Vec::new();
        for (token, handle) in &self.bots {
            if handle.is_finished() {
                bots_to_restart.push(token.clone());
            }
        }

        for (token, bot_id) in bots_to_restart {
            tracing::warn!("Bot with token {} has finished, restarting...", token);
            self.bots.remove(&(token.clone(), bot_id));
            let app_state = self.app_state.clone();
            let bot_token = token.clone();
            let handle = tokio::spawn(async move {
                let api = Arc::new(
                    BackendApi::new(
                        &app_state.config.backend_api_url,
                        &app_state.config.service_api_key,
                        Some(bot_id),
                    )
                    .expect("Failed to create BackendApi"),
                );
                if let Err(e) = run_bot(bot_token, app_state, api).await {
                    tracing::error!("Bot exited with error: {}", e);
                }
            });
            self.bots.insert((token, bot_id), handle);
        }
    }
}
