use crate::{
    api::backend_api::BackendApi,
    bot::{
        MyDialogue,
        keyboards::{
            back_to_main_menu::back_to_main_menu_inline_keyboard,
            my_payments::my_payments_inline_keyboard,
        },
        utils::{MsgBy, edit_msg},
    },
    errors::AppResult,
};
use shared_dtos::invoice::{InvoiceStatus, PaymentInvoiceBotResponse};
use std::sync::Arc;
use teloxide::{
    prelude::Bot,
    types::CallbackQuery,
    utils::html::{bold, escape, underline},
};

pub async fn my_payments_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let telegram_id = dialogue.chat_id().0;
    let invoices = match api_client.get_customer_invoices(telegram_id).await {
        Ok(invoices) => invoices,
        Err(_) => {
            edit_msg(
                &api_client,
                &dialogue,
                &bot,
                &MsgBy::CallbackQuery(&q),
                "Не удалось загрузить историю платежей. Попробуйте позже.",
                None,
                back_to_main_menu_inline_keyboard(),
            )
            .await?;
            return Ok(());
        }
    };
    let pending_payments = invoices
        .items
        .iter()
        .filter(|i| i.status == InvoiceStatus::Pending)
        .collect::<Vec<_>>();
    let completed_payments = invoices
        .items
        .iter()
        .filter(|i| i.status == InvoiceStatus::Completed)
        .collect::<Vec<_>>();
    let mut text = bold("🧾 Мои платежи\n\n");
    if !pending_payments.is_empty() {
        text.push_str(&underline("Активные платежи:\n"));
        for payment in &pending_payments {
            text.push_str(&format!("• {}\n\n", format_payment_info(payment)));
        }
        text.push('\n');
    } else {
        text.push_str("У вас нет активных счетов для оплаты.\n\n");
    }

    if !completed_payments.is_empty() {
        text.push_str(&underline("История операций:\n"));
        for payment in &completed_payments {
            text.push_str(&format!("• {}\n\n", format_payment_info(payment)));
        }
    }

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        &text,
        None,
        my_payments_inline_keyboard(&pending_payments),
    )
    .await?;
    Ok(())
}

fn format_payment_info(payment: &PaymentInvoiceBotResponse) -> String {
    let token = escape(&payment.gateway_invoice_id);
    format!(
        "<b>Платеж #{}:</b> <code>{}</code> ₽\n\
         <b>Дата:</b> {}\n\
         <b>Токен:</b> <code>{}</code>",
        payment.id,
        payment.amount,
        payment.created_at.format("%d.%m.%Y"),
        token
    )
}
