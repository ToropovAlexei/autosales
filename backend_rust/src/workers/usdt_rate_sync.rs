use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde::Deserialize;
use std::sync::Arc;
use tokio::time::{Duration, interval};

use crate::{
    services::settings::{SettingsServiceTrait, UpdateSettingsCommand},
    state::AppState,
};

#[derive(Debug, Deserialize)]
struct PriceResponse {
    price: f64,
}

pub async fn usdt_rate_sync_task(app_state: Arc<AppState>) {
    tracing::info!("Starting USDT Rate Sync Task");
    let mut interval = interval(Duration::from_mins(30));

    loop {
        interval.tick().await;
        tracing::info!("Running USDT Rate Sync Task...");
        let response = app_state
            .client
            .get("https://api.binance.com/api/v3/ticker/price?symbol=USDTRUB")
            .send()
            .await;
        let response = match response {
            Ok(response) => response,
            Err(e) => {
                tracing::error!("Error fetching USDT rate: {}", e);
                continue;
            }
        };
        let response = match response.json::<PriceResponse>().await {
            Ok(response) => response,
            Err(e) => {
                tracing::error!("Error parsing USDT rate: {}", e);
                continue;
            }
        };
        let rate = Decimal::from_f64(response.price);
        let rate = match rate {
            Some(rate) => rate,
            None => {
                tracing::error!("Error parsing USDT rate");
                continue;
            }
        };
        if let Err(err) = app_state
            .settings_service
            .update(UpdateSettingsCommand {
                usdt_rate_rub: Some(rate),
                updated_by: 1, // System
                ..Default::default()
            })
            .await
        {
            tracing::error!("Error updating USDT rate: {}", err);
        }
    }
}
