use crate::bot_manager::BotManager;
use std::{sync::Arc, time::Duration};

pub struct HealthChecker {
    bot_manager: Arc<tokio::sync::Mutex<BotManager>>,
}

impl HealthChecker {
    pub fn new(bot_manager: Arc<tokio::sync::Mutex<BotManager>>) -> Self {
        Self { bot_manager }
    }

    pub async fn start(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let mut bot_manager = self.bot_manager.lock().await;
            bot_manager.check_bots_health().await;
        }
    }
}
