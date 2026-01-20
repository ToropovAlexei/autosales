use std::sync::Arc;

use chrono::Utc;
use serde_json::json;
use tokio::time::{Duration, interval};

use crate::{
    models::{broadcast::BroadcastStatus, customer::CustomerListQuery},
    presentation::admin::dtos::broadcast::JsonRawListQuery,
    services::{
        broadcast::{BroadcastServiceTrait, UpdateBroadcastCommand},
        customer::CustomerServiceTrait,
        notification_service::{
            DispatchMessage, DispatchMessageCommand, GenericMessage, NotificationServiceTrait,
        },
    },
    state::AppState,
};

pub async fn broadcasts_task(app_state: Arc<AppState>) {
    tracing::info!("Starting broadcasts task");
    let mut interval = interval(Duration::from_mins(1));

    loop {
        interval.tick().await;
        tracing::info!("Running broadcasts task...");
        let ready_broadcasts = match app_state.broadcast_service.get_ready_broadcasts().await {
            Ok(broadcasts) => broadcasts,
            Err(e) => {
                tracing::error!("Error getting ready broadcasts: {}", e);
                continue;
            }
        };

        for broadcast in ready_broadcasts {
            let json_val = broadcast.filters.unwrap_or_default();
            let raw_query: JsonRawListQuery = match serde_json::from_value(json_val) {
                Ok(q) => q,
                Err(e) => {
                    tracing::error!("Error parsing raw broadcast filters: {e}");
                    mark_broadcast_as_failed(broadcast.id, &app_state).await;
                    continue;
                }
            };

            let mut list_query = match CustomerListQuery::try_from_json(raw_query) {
                Ok(q) => q,
                Err(e) => {
                    tracing::error!("Error parsing broadcast filters: {e}");
                    mark_broadcast_as_failed(broadcast.id, &app_state).await;
                    continue;
                }
            };

            list_query.pagination.page = 1;
            list_query.pagination.page_size = 100000; // TODO: Make this configurable
            let customers = match app_state.customer_service.get_list(list_query).await {
                Ok(customers) => customers,
                Err(e) => {
                    tracing::error!("Error getting broadcast customers: {e}");
                    mark_broadcast_as_failed(broadcast.id, &app_state).await;
                    continue;
                }
            };

            if customers.items.is_empty() {
                mark_broadcast_as_failed(broadcast.id, &app_state).await;
                tracing::error!("No customers found for broadcast");
                continue;
            }

            if let Err(e) = app_state
                .broadcast_service
                .update(UpdateBroadcastCommand {
                    id: broadcast.id,
                    status: Some(BroadcastStatus::InProgress),
                    updated_by: Some(1), // System
                    started_at: Some(Some(Utc::now())),
                    content_image_id: None,
                    content_text: None,
                    ctx: None,
                    filters: None,
                    scheduled_for: None,
                    finished_at: None,
                    statistics: None,
                })
                .await
            {
                tracing::error!("Error updating broadcast status: {e}");
                continue;
            }

            let mut sent = 0;
            let mut failed = 0;

            tracing::info!("Sending broadcast to {} customers", customers.items.len());

            for customer in customers.items {
                match app_state
                    .notification_service
                    .dispatch_message(DispatchMessageCommand {
                        // TODO Last seen with bot may be old if we created new bot
                        bot_id: customer.last_seen_with_bot,
                        telegram_id: customer.telegram_id,
                        message: DispatchMessage::GenericMessage(GenericMessage {
                            message: broadcast.content_text.clone().unwrap_or_default(),
                            image_id: broadcast.content_image_id,
                        }),
                    })
                    .await
                {
                    Ok(_) => sent += 1,
                    Err(e) => {
                        tracing::error!("Error sending broadcast message: {e}");
                        failed += 1;
                    }
                }
                // Because of the telegram rate limit https://core.telegram.org/bots/faq#my-bot-is-hitting-limits-how-do-i-avoid-this
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            tracing::info!("Broadcast sent to {} customers", sent);
            tracing::info!("Broadcast failed to send to {} customers", failed);

            if let Err(e) = app_state
                .broadcast_service
                .update(UpdateBroadcastCommand {
                    id: broadcast.id,
                    status: Some(BroadcastStatus::Completed),
                    updated_by: Some(1), // System
                    finished_at: Some(Some(Utc::now())),
                    statistics: Some(Some(json!({
                        "sent": sent,
                        "failed": failed,
                    }))),
                    content_image_id: None,
                    content_text: None,
                    ctx: None,
                    filters: None,
                    scheduled_for: None,
                    started_at: None,
                })
                .await
            {
                tracing::error!("Error updating broadcast status: {e}");
                continue;
            };
        }
    }
}

pub async fn mark_broadcast_as_failed(broadcast_id: i64, app_state: &Arc<AppState>) {
    if let Err(e) = app_state
        .broadcast_service
        .update(UpdateBroadcastCommand {
            id: broadcast_id,
            status: Some(BroadcastStatus::Failed),
            updated_by: Some(1), // System
            finished_at: Some(Some(Utc::now())),
            content_image_id: None,
            content_text: None,
            ctx: None,
            filters: None,
            scheduled_for: None,
            started_at: None,
            statistics: None,
        })
        .await
    {
        tracing::error!("Error marking broadcast as failed: {e}");
    };
}
