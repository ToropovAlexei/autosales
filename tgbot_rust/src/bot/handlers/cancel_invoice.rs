use crate::bot::utils::{MsgBy, edit_msg};
use crate::{
    api::backend_api::BackendApi, bot::MyDialogue,
    bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard, errors::AppResult,
};
use chrono::{Duration, Utc};
use shared_dtos::invoice::InvoiceStatus;
use std::sync::Arc;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::{prelude::Bot, types::CallbackQuery};

pub async fn cancel_invoice_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    invoice_id: i64,
) -> AppResult<()> {
    let telegram_id = match q.chat_id() {
        Some(chat_id) => chat_id.0,
        None => return Ok(()),
    };
    let customer_invoices = api_client.get_customer_invoices(telegram_id).await?;
    let total_cancelled = customer_invoices
        .items
        .iter()
        .filter(|i| {
            i.created_at > Utc::now() - Duration::days(1) && i.status == InvoiceStatus::Cancelled
        })
        .count()
        + 1;

    let msg = match api_client.cancel_invoice(invoice_id).await {
        Ok(_) => {
            format!("Заявка отменена\n<u>Ваш лимит на отмену платежа: {total_cancelled}/3 раз</u>")
        }
        Err(_) => "Не удалось отменить платеж. Попробуйте позже.".to_string(),
    };

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        &msg,
        None,
        back_to_main_menu_inline_keyboard(),
    )
    .await?;

    Ok(())
}
