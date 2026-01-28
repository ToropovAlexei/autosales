use crate::AppState;
use crate::api::backend_api::BackendApi;
use crate::bot::{BotUsername, run_bot};
use crate::errors::AppError;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::interval;

pub struct BotManager {
    bots: Arc<Mutex<HashMap<i64, JoinHandle<()>>>>,
    app_state: Arc<AppState>,
}

impl BotManager {
    pub fn new(app_state: Arc<AppState>) -> Self {
        let bots = Arc::new(Mutex::new(HashMap::new()));

        Self { bots, app_state }
    }

    pub async fn run(&self) {
        self.start_bots().await;
    }

    pub async fn start_bots(&self) {
        let mut interval = interval(Duration::from_mins(1));
        loop {
            interval.tick().await;
            let all_bots = match self.app_state.api.get_bots().await {
                Ok(bots) => bots,
                Err(e) => {
                    tracing::error!(error = %e, "Failed to get bots");
                    continue;
                }
            };
            let fallback_bot = all_bots
                .items
                .iter()
                .find(|b| b.is_active && b.owner_id.is_none() && !b.is_primary)
                .map(|b| BotUsername(b.username.clone()));

            let guard = self.bots.lock().await;
            let running_bots = guard.keys().copied().collect::<HashSet<_>>();
            // Dont forget to drop the lock. Else we will have a deadlock as we start and stop bots later
            drop(guard);

            let all_primary_bots = all_bots
                .items
                .iter()
                .filter_map(|b| if b.is_primary { Some(b.id) } else { None })
                .collect::<HashSet<_>>();

            let bots_to_start = all_bots
                .items
                .iter()
                .filter(|b| b.is_primary && !running_bots.contains(&b.id))
                .collect::<Vec<_>>();

            let bots_to_stop = running_bots
                .difference(&all_primary_bots)
                .collect::<Vec<_>>();

            for bot in bots_to_start {
                let fallback_bot_name = if bot.owner_id.is_some() {
                    None
                } else {
                    fallback_bot.clone()
                };
                self.start_bot(bot.token.clone(), bot.id, fallback_bot_name)
                    .await;
            }

            for bot_id in bots_to_stop {
                self.stop_bot(*bot_id).await;
            }
        }
    }

    async fn start_bot(
        &self,
        bot_token: String,
        bot_id: i64,
        fallback_bot_name: Option<BotUsername>,
    ) {
        let app_state = self.app_state.clone();
        let bots = self.bots.clone();
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
                match run_bot(
                    token_clone,
                    bot_id,
                    app_state.deref().clone(),
                    api,
                    fallback_bot_name,
                )
                .await
                {
                    Ok(_) => break,
                    Err(e) => {
                        match e {
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
                        }
                    }
                }
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
            bots.lock().await.remove(&bot_id);
        });
        self.bots.lock().await.insert(bot_id, handle);
    }

    async fn stop_bot(&self, bot_id: i64) {
        if let Some(handle) = self.bots.lock().await.remove(&bot_id) {
            handle.abort();
        }
    }
}
