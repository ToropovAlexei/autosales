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

async fn run_subscription_expiry_notifications_once(
    user_subscription_service: &dyn UserSubscriptionServiceTrait,
    notification_service: &dyn NotificationServiceTrait,
    window_hours: i64,
) {
    if window_hours <= 0 {
        tracing::warn!(
            "[Subscription expiry notifications task]: Disabled by config: subscription_expiry_notification_window_hours <= 0"
        );
        return;
    }

    let expiring = match user_subscription_service
        .get_expiring_for_notification(window_hours)
        .await
    {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!(
                "[Subscription expiry notifications task]: Failed to load expiring subscriptions: {err}"
            );
            return;
        }
    };

    if expiring.is_empty() {
        return;
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

        match notification_service.dispatch_message(payload).await {
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
        if let Err(err) = user_subscription_service
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
        run_subscription_expiry_notifications_once(
            app_state.user_subscription_service.as_ref(),
            app_state.notification_service.as_ref(),
            app_state
                .config
                .subscription_expiry_notification_window_hours,
        )
        .await;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use shared_dtos::notification::{DispatchAdminMessage, DispatchMessage};

    use crate::{
        errors::api::{ApiError, ApiResult},
        models::user_subscription::{
            NewUserSubscription, UserSubscriptionEnrichedRow,
            UserSubscriptionExpiryNotificationRow, UserSubscriptionRow,
        },
        services::{
            notification_service::NotificationServiceTrait,
            user_subscription::UserSubscriptionServiceTrait,
        },
    };

    use super::run_subscription_expiry_notifications_once;

    #[derive(Default)]
    struct MockUserSubscriptionService {
        expiring: Mutex<Vec<UserSubscriptionExpiryNotificationRow>>,
        mark_calls: Mutex<Vec<Vec<i64>>>,
    }

    #[async_trait]
    impl UserSubscriptionServiceTrait for MockUserSubscriptionService {
        async fn create(
            &self,
            _user_subscription: NewUserSubscription,
        ) -> ApiResult<UserSubscriptionRow> {
            panic!("create is not used in this test");
        }

        async fn get_for_customer(&self, _id: i64) -> ApiResult<Vec<UserSubscriptionEnrichedRow>> {
            panic!("get_for_customer is not used in this test");
        }

        async fn get_expiring_for_notification(
            &self,
            _within_hours: i64,
        ) -> ApiResult<Vec<UserSubscriptionExpiryNotificationRow>> {
            Ok(self.expiring.lock().unwrap().clone())
        }

        async fn mark_expiry_notification_sent(&self, subscription_ids: &[i64]) -> ApiResult<u64> {
            self.mark_calls
                .lock()
                .unwrap()
                .push(subscription_ids.to_vec());
            Ok(subscription_ids.len() as u64)
        }
    }

    #[derive(Default)]
    struct MockNotificationService {
        sent_payloads: Mutex<Vec<shared_dtos::notification::DispatchMessagePayload>>,
        fail_for_telegram_ids: Mutex<Vec<i64>>,
    }

    #[async_trait]
    impl NotificationServiceTrait for MockNotificationService {
        async fn dispatch_message(
            &self,
            payload: shared_dtos::notification::DispatchMessagePayload,
        ) -> ApiResult<()> {
            let should_fail = match &payload.message {
                DispatchMessage::SubscriptionExpiringNotification { .. } => self
                    .fail_for_telegram_ids
                    .lock()
                    .unwrap()
                    .contains(&payload.telegram_id),
                _ => false,
            };
            self.sent_payloads.lock().unwrap().push(payload);
            if should_fail {
                return Err(ApiError::InternalServerError(
                    "simulated dispatch error".to_string(),
                ));
            }
            Ok(())
        }

        async fn dispatch_admin_message(&self, _payload: DispatchAdminMessage) -> ApiResult<()> {
            Ok(())
        }
    }

    fn make_row(id: i64, telegram_id: i64) -> UserSubscriptionExpiryNotificationRow {
        UserSubscriptionExpiryNotificationRow {
            id,
            expires_at: Utc::now() + Duration::hours(2),
            product_name: Some(format!("prod-{id}")),
            telegram_id,
            last_seen_with_bot: 100,
        }
    }

    #[tokio::test]
    async fn test_run_once_marks_only_successful_dispatches() {
        let user_sub = Arc::new(MockUserSubscriptionService::default());
        let notification = Arc::new(MockNotificationService::default());

        {
            let mut expiring = user_sub.expiring.lock().unwrap();
            expiring.push(make_row(1, 1001));
            expiring.push(make_row(2, 1002));
            expiring.push(make_row(3, 1003));
        }
        {
            let mut fail_ids = notification.fail_for_telegram_ids.lock().unwrap();
            fail_ids.push(1002);
        }

        run_subscription_expiry_notifications_once(user_sub.as_ref(), notification.as_ref(), 24)
            .await;

        let mark_calls = user_sub.mark_calls.lock().unwrap();
        assert_eq!(mark_calls.len(), 1);
        assert_eq!(mark_calls[0], vec![1, 3]);
    }

    #[tokio::test]
    async fn test_run_once_disabled_window_does_nothing() {
        let user_sub = Arc::new(MockUserSubscriptionService::default());
        let notification = Arc::new(MockNotificationService::default());

        {
            let mut expiring = user_sub.expiring.lock().unwrap();
            expiring.push(make_row(1, 1001));
        }

        run_subscription_expiry_notifications_once(user_sub.as_ref(), notification.as_ref(), 0)
            .await;

        assert!(notification.sent_payloads.lock().unwrap().is_empty());
        assert!(user_sub.mark_calls.lock().unwrap().is_empty());
    }
}
