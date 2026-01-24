use chrono::Utc;
use rust_decimal::prelude::ToPrimitive;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::time::{Duration, interval};

use crate::{
    infrastructure::external::payment::autosales_platform::{
        AutosalesPlatformPaymentsProviderTrait, dto::AutosalesPlatformOrderStatusType,
    },
    models::{
        customer::CustomerRow,
        payment_invoice::{InvoiceStatus, PaymentInvoiceRow},
    },
    services::{
        customer::CustomerServiceTrait,
        notification_service::{DispatchMessage, DispatchMessageCommand, NotificationServiceTrait},
        payment_invoice::{PaymentInvoiceServiceTrait, UpdatePaymentInvoiceCommand},
        payment_processing_service::PaymentProcessingServiceTrait,
    },
    state::AppState,
};

// TODO Need to refactor this task

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

        tracing::info!(
            "[Pending payments task]: Found {} pending payments",
            pending_invoices.len()
        );

        let polled_statuses = {
            let mut statuses = HashMap::with_capacity(pending_invoices.len());
            for invoice in pending_invoices.iter() {
                if let Ok(order) = app_state
                    .platform_payments_provider
                    .get_order_status(invoice.gateway_invoice_id.clone())
                    .await
                {
                    statuses.insert(invoice.id, InvoiceStatus::from(order.status));
                }
            }
            statuses
        };

        tracing::info!(
            "[Pending payments task]: Polled {} pending payments",
            polled_statuses.len()
        );

        notify_completed_payments(&app_state, &pending_invoices, &polled_statuses).await;
        notify_pending_payments(
            &app_state,
            &pending_invoices,
            &polled_statuses,
            &customers_by_id,
        )
        .await;
        request_receipts(
            &app_state,
            &pending_invoices,
            &polled_statuses,
            &customers_by_id,
        )
        .await;
        notify_contact_with_support(
            &app_state,
            &pending_invoices,
            &polled_statuses,
            &customers_by_id,
        )
        .await;
        update_invoices(&app_state, &pending_invoices, &polled_statuses).await;

        tracing::info!("[Pending payments task]: Finished");
    }
}

async fn notify_completed_payments(
    app_state: &Arc<AppState>,
    pending_invoices: &[PaymentInvoiceRow],
    polled_statuses: &HashMap<i64, InvoiceStatus>,
) {
    let completed_invoices = pending_invoices
        .iter()
        .filter(|i| {
            polled_statuses
                .get(&i.id)
                .unwrap_or(&InvoiceStatus::Pending)
                == &InvoiceStatus::Completed
        })
        .collect::<Vec<_>>();

    let mut notifications_sent = Vec::with_capacity(completed_invoices.len());
    for invoice in completed_invoices {
        match app_state
            .payment_processing_service
            .handle_payment_success(invoice.order_id)
            .await
        {
            Ok(_) => {
                notifications_sent.push(invoice.id);
            }
            Err(e) => {
                tracing::error!("[Pending payments task]: Failed to handle payment success: {e}");
            }
        }
    }
    if !notifications_sent.is_empty() {
        tracing::info!(
            "[Pending payments task]: Sent {} payment success notifications",
            notifications_sent.len()
        );
    }
}

async fn notify_pending_payments(
    app_state: &Arc<AppState>,
    pending_invoices: &[PaymentInvoiceRow],
    polled_statuses: &HashMap<i64, InvoiceStatus>,
    customers_by_id: &HashMap<i64, CustomerRow>,
) {
    let invoices_to_notify_about_troubles = pending_invoices
        .iter()
        .filter(|i| {
            i.status == InvoiceStatus::Pending
                && polled_statuses
                    .get(&i.id)
                    .unwrap_or(&InvoiceStatus::Pending)
                    == &InvoiceStatus::Pending
        })
        .collect::<Vec<_>>();

    let mut notifications_sent = Vec::with_capacity(invoices_to_notify_about_troubles.len());
    for invoice in invoices_to_notify_about_troubles {
        let customer = match customers_by_id.get(&invoice.customer_id) {
            Some(c) => c,
            None => continue,
        };
        match app_state
            .notification_service
            .dispatch_message(DispatchMessageCommand {
                message: DispatchMessage::InvoiceTroublesNotification {
                    invoice_id: invoice.id,
                    amount: invoice.original_amount.to_f64().unwrap_or_default(),
                },
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
}

async fn request_receipts(
    app_state: &Arc<AppState>,
    pending_invoices: &[PaymentInvoiceRow],
    polled_statuses: &HashMap<i64, InvoiceStatus>,
    customers_by_id: &HashMap<i64, CustomerRow>,
) {
    let to_request_receipt = pending_invoices
        .iter()
        .filter(|i| {
            i.status == InvoiceStatus::Pending
                && polled_statuses
                    .get(&i.id)
                    .unwrap_or(&InvoiceStatus::Pending)
                    == &InvoiceStatus::AwaitingReceipt
        })
        .collect::<Vec<_>>();
    let mut notifications_sent = Vec::with_capacity(to_request_receipt.len());
    for invoice in to_request_receipt {
        let customer = match customers_by_id.get(&invoice.customer_id) {
            Some(c) => c,
            None => continue,
        };
        match app_state
            .notification_service
            .dispatch_message(DispatchMessageCommand {
                message: DispatchMessage::RequestReceiptNotification {
                    invoice_id: invoice.id,
                },
                telegram_id: customer.telegram_id,
                bot_id: customer.last_seen_with_bot,
            })
            .await
        {
            Ok(_) => notifications_sent.push(invoice.id),
            Err(e) => {
                tracing::error!(
                    "[Pending payments task]: Failed to send request receipt notification: {e}"
                );
            }
        }
    }
    if !notifications_sent.is_empty() {
        tracing::info!(
            "[Pending payments task]: Sent {} request receipt notifications",
            notifications_sent.len()
        );
    }
    for invoice_id in notifications_sent {
        if let Err(e) = app_state
            .payment_invoice_service
            .update(UpdatePaymentInvoiceCommand {
                id: invoice_id,
                status: Some(InvoiceStatus::AwaitingReceipt),
                ..Default::default()
            })
            .await
        {
            tracing::error!("[Pending payments task]: Failed to update payment invoice: {e}")
        }
    }
}

async fn notify_contact_with_support(
    app_state: &Arc<AppState>,
    pending_invoices: &[PaymentInvoiceRow],
    polled_statuses: &HashMap<i64, InvoiceStatus>,
    customers_by_id: &HashMap<i64, CustomerRow>,
) {
    let to_notify = pending_invoices
        .iter()
        .filter(|i| {
            i.status == InvoiceStatus::ReceiptSubmitted
                && polled_statuses
                    .get(&i.id)
                    .unwrap_or(&InvoiceStatus::ReceiptSubmitted)
                    == &InvoiceStatus::Disputed
        })
        .collect::<Vec<_>>();
    let mut notifications_sent = Vec::with_capacity(to_notify.len());
    for invoice in to_notify {
        let customer = match customers_by_id.get(&invoice.customer_id) {
            Some(c) => c,
            None => continue,
        };
        match app_state
            .notification_service
            .dispatch_message(DispatchMessageCommand {
                message: DispatchMessage::GenericMessage {
                    // TODO replace @operator_contact_placeholder
                    message: "Мы не смогли увидеть Ваш платеж. Пожалуйста свяжитесь с оператором: @operator_contact_placeholder".to_string(),
                    image_id: None,
                },
                telegram_id: customer.telegram_id,
                bot_id: customer.last_seen_with_bot,
            })
            .await
        {
            Ok(_) => notifications_sent.push(invoice.id),
            Err(e) => {
                tracing::error!(
                    "[Pending payments task]: Failed to send contact support notification: {e}"
                );
            }
        }
    }
    if !notifications_sent.is_empty() {
        tracing::info!(
            "[Pending payments task]: Sent {} contact support notifications",
            notifications_sent.len()
        );
    }
    for invoice_id in notifications_sent {
        if let Err(e) = app_state
            .payment_invoice_service
            .update(UpdatePaymentInvoiceCommand {
                id: invoice_id,
                status: Some(InvoiceStatus::Disputed),
                ..Default::default()
            })
            .await
        {
            tracing::error!("[Pending payments task]: Failed to update payment invoice: {e}")
        }
    }
}

async fn update_invoices(
    app_state: &Arc<AppState>,
    pending_invoices: &[PaymentInvoiceRow],
    polled_statuses: &HashMap<i64, InvoiceStatus>,
) {
    let to_update = pending_invoices
        .iter()
        .map(|i| {
            (
                i.id,
                i.status,
                polled_statuses
                    .get(&i.id)
                    .copied()
                    .unwrap_or(InvoiceStatus::Pending),
            )
        })
        .filter(|(_, status, polled_status)| {
            polled_status != status
                && (polled_status == &InvoiceStatus::Failed
                    || polled_status == &InvoiceStatus::Expired
                    || polled_status == &InvoiceStatus::Cancelled)
        })
        .collect::<Vec<_>>();
    for (invoice_id, _, polled_status) in to_update {
        if let Err(e) = app_state
            .payment_invoice_service
            .update(UpdatePaymentInvoiceCommand {
                id: invoice_id,
                status: Some(polled_status),
                ..Default::default()
            })
            .await
        {
            tracing::error!("[Pending payments task]: Failed to update payment invoice: {e}")
        }
    }
}

impl From<AutosalesPlatformOrderStatusType> for InvoiceStatus {
    fn from(status: AutosalesPlatformOrderStatusType) -> Self {
        match status {
            AutosalesPlatformOrderStatusType::TraderSuccess
            | AutosalesPlatformOrderStatusType::MerchSuccess
            | AutosalesPlatformOrderStatusType::SystemTimerEndMerchProcessSuccess
            | AutosalesPlatformOrderStatusType::SystemTimerEndMerchCheckDownSuccess
            | AutosalesPlatformOrderStatusType::AdminAppealSuccess => InvoiceStatus::Completed,
            AutosalesPlatformOrderStatusType::TraderCheckQuery => InvoiceStatus::AwaitingReceipt,
            AutosalesPlatformOrderStatusType::TraderAppeal => InvoiceStatus::Disputed,
            AutosalesPlatformOrderStatusType::SystemTimerEndMerchInitializedCancel => {
                InvoiceStatus::Cancelled
            }
            AutosalesPlatformOrderStatusType::OrderCancel => InvoiceStatus::Cancelled,
            AutosalesPlatformOrderStatusType::MerchCancel => InvoiceStatus::Cancelled,
            AutosalesPlatformOrderStatusType::SystemTimerEndTraderCheckQueryCancel => {
                InvoiceStatus::Cancelled
            }
            AutosalesPlatformOrderStatusType::AdminAppealCancel => InvoiceStatus::Failed,
        }
    }
}
