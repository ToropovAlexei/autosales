use chrono::Utc;
use rust_decimal::prelude::ToPrimitive;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::time::{Duration, interval};

use crate::{
    services::{
        customer::CustomerServiceTrait,
        notification_service::{
            DispatchMessage, DispatchMessageCommand, InvoiceTroublesNotification,
            NotificationServiceTrait,
        },
        payment_invoice::PaymentInvoiceServiceTrait,
    },
    state::AppState,
};

pub async fn pending_payments_task(app_state: Arc<AppState>) {
    tracing::info!("[Pending payments task]: Starting");
    let mut interval = interval(Duration::from_mins(1));

    loop {
        interval.tick().await;
        tracing::info!("[Pending payments task]: Running...");
        match app_state
            .payment_invoice_service
            .expire_old_invoices()
            .await
        {
            Ok(res) => {
                if res > 0 {
                    tracing::info!("[Pending payments task]: Expired pending payments {res} mark")
                }
            }
            Err(e) => {
                tracing::error!("[Pending payments task]: Failed to expire old payments: {e}")
            }
        };
        let pending_invoices = match app_state
            .payment_invoice_service
            .get_pending_invoices(
                Utc::now() - Duration::from_mins(app_state.config.payment_notification_minutes),
            )
            .await
        {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("[Pending payments task]: Failed to get pending payments: {e}");
                continue;
            }
        };

        let customer_ids = pending_invoices
            .iter()
            .map(|i| i.customer_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let customers_by_id = match app_state
            .customer_service
            .get_list_by_ids(&customer_ids)
            .await
        {
            Ok(res) => res
                .into_iter()
                .map(|c| (c.id, c))
                .collect::<HashMap<_, _>>(),
            Err(e) => {
                tracing::error!("[Pending payments task]: Failed to get customer by ids: {e}");
                continue;
            }
        };

        // TODO Poll from external service

        tracing::info!(
            "[Pending payments task]: Found {} pending payments",
            pending_invoices.len()
        );
        let mut notifications_sent = Vec::with_capacity(pending_invoices.len());
        for invoice in pending_invoices {
            let customer = match customers_by_id.get(&invoice.customer_id) {
                Some(c) => c,
                None => continue,
            };
            match app_state
                .notification_service
                .dispatch_message(DispatchMessageCommand {
                    message: DispatchMessage::InvoiceTroublesNotification(
                        InvoiceTroublesNotification {
                            invoice_id: invoice.id,
                            amount: invoice.original_amount.to_f64().unwrap_or_default(),
                        },
                    ),
                    telegram_id: customer.telegram_id,
                    bot_id: customer.last_seen_with_bot,
                })
                .await
            {
                Ok(_) => notifications_sent.push(invoice.id),
                Err(e) => {
                    tracing::error!(
                        "[Pending payments task]: Failed to send payment notification: {e}"
                    );
                }
            }
        }
        if !notifications_sent.is_empty() {
            tracing::info!(
                "[Pending payments task]: Sent {} payment notifications",
                notifications_sent.len()
            );
        }
        match app_state
            .payment_invoice_service
            .mark_invoices_notified(&notifications_sent)
            .await
        {
            Ok(res) => {
                if res > 0 {
                    tracing::info!("[Pending payments task]: Marked {} payments as sent", res)
                }
            }
            Err(e) => {
                tracing::error!("[Pending payments task]: Failed to mark payments as sent: {e}")
            }
        }
        tracing::info!("[Pending payments task]: Finished");
    }
}
