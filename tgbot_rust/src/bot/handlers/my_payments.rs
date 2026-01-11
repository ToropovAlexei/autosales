use crate::{
    api::backend_api::BackendApi,
    bot::{
        MyDialogue,
        keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard,
        utils::{MsgBy, edit_msg},
    },
    errors::AppResult,
    models::payment::InvoiceStatus,
};
use std::sync::Arc;
use teloxide::{prelude::Bot, types::CallbackQuery, utils::html::bold};

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
    let text_rows = vec![bold("🧾 Мои платежи")];
    Ok(())
}
