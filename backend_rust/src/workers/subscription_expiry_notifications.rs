use std::sync::Arc;

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
        let mut sent_ids = Vec::new();

        for sub in expiring {
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
                    sent_ids.push(sub.id);
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
            if let Err(err) = app_state
                .user_subscription_service
                .mark_expiry_notification_sent(&sent_ids)
                .await
            {
                tracing::error!(
                    "[Subscription expiry notifications task]: Failed to mark notifications sent: {}",
                    err
                );
            }
            tracing::info!(
                "[Subscription expiry notifications task]: Sent {} notifications",
                sent
            );
        }
    }
}
