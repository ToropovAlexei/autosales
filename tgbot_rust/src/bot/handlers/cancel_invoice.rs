use crate::bot::utils::{MsgBy, edit_msg};
use crate::{
    api::backend_api::BackendApi, bot::MyDialogue,
    bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard, errors::AppResult,
};
use std::sync::Arc;
use teloxide::{prelude::Bot, types::CallbackQuery};

pub async fn cancel_invoice_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    invoice_id: i64,
) -> AppResult<()> {
    let msg = match api_client.cancel_invoice(invoice_id).await {
        // TODO WIP
        Ok(_) => "Заявка отменена\nВаш лимит на отмену платежа: WIP/3 раз",
        Err(_) => "Не удалось отменить платеж. Попробуйте позже.",
    };

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        msg,
        None,
        back_to_main_menu_inline_keyboard(),
    )
    .await?;

    Ok(())
}
