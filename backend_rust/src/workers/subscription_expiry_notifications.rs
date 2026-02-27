use std::sync::Arc;

use chrono::{Duration, Utc};
use deadpool_redis::redis::AsyncCommands;
use shared_dtos::notification::{DispatchMessage, DispatchMessagePayload};
use tokio::time::{Duration as TokioDuration, interval};

use crate::{
    services::{
        notification_service::NotificationServiceTrait,
        user_subscription::UserSubscriptionServiceTrait,
    },
    state::AppState,
};

pub async fn subscription_expiry_notifications_task(app_state: Arc<AppState>) {
    tracing::info!("[Subscription expiry notifications task]: Starting");
    let mut interval = interval(TokioDuration::from_secs(
        app_state
            .config
            .subscription_expiry_notification_poll_interval_seconds,
    ));

    loop {
        interval.tick().await;
        tracing::info!("[Subscription expiry notifications task]: Running...");

        if app_state
            .config
            .subscription_expiry_notification_window_hours
            <= 0
        {
            tracing::warn!(
                "[Subscription expiry notifications task]: Disabled by config: subscription_expiry_notification_window_hours <= 0"
            );
            continue;
        }

        let expiring = match app_state
            .user_subscription_service
            .get_expiring_for_notification(
                app_state
                    .config
                    .subscription_expiry_notification_window_hours,
            )
            .await
        {
            Ok(rows) => rows,
            Err(err) => {
                tracing::error!(
                    "[Subscription expiry notifications task]: Failed to load expiring subscriptions: {err}"
                );
                continue;
            }
        };

        if expiring.is_empty() {
            continue;
        }

        let mut sent = 0usize;

        for sub in expiring {
            let dedupe_key = format!(
                "subscription-expiry-notified:{}:{}",
                sub.id,
                sub.expires_at.timestamp()
            );

            let should_notify = match app_state.redis_pool.get().await {
                Ok(mut conn) => {
                    let was_set: redis::RedisResult<bool> = conn.set_nx(&dedupe_key, "1").await;
                    match was_set {
                        Ok(true) => {
                            let ttl_seconds = ((sub.expires_at - Utc::now()) + Duration::days(2))
                                .num_seconds()
                                .max(60);
                            let _: redis::RedisResult<bool> =
                                conn.expire(&dedupe_key, ttl_seconds).await;
                            true
                        }
                        Ok(false) => false,
                        Err(err) => {
                            tracing::warn!(
                                "[Subscription expiry notifications task]: Failed set_nx for subscription {}: {}",
                                sub.id,
                                err
                            );
                            false
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        "[Subscription expiry notifications task]: Failed to acquire redis connection: {}",
                        err
                    );
                    false
                }
            };

            if !should_notify {
                continue;
            }

            let payload = DispatchMessagePayload {
                message: DispatchMessage::SubscriptionExpiringNotification {
                    expires_at: sub.expires_at,
                    product_name: sub.product_name.clone(),
                },
                telegram_id: sub.telegram_id,
                bot_id: sub.last_seen_with_bot,
            };

            match app_state
                .notification_service
                .dispatch_message(payload)
                .await
            {
                Ok(_) => {
                    sent += 1;
                }
                Err(err) => {
                    tracing::error!(
                        "[Subscription expiry notifications task]: Failed to dispatch notification for subscription {}: {}",
                        sub.id,
                        err
                    );
                }
            }
        }

        if sent > 0 {
            tracing::info!(
                "[Subscription expiry notifications task]: Sent {} notifications",
                sent
            );
        }
    }
}
