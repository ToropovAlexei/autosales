use crate::bot::keyboards::my_orders_menu::my_orders_inline_keyboard;
use crate::bot::utils::edit_msg;
use crate::{
    api::backend_api::BackendApi, bot::MyDialogue,
    bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard, errors::AppResult,
};
use std::sync::Arc;
use teloxide::{
    dispatching::dialogue::GetChatId, prelude::Bot, types::CallbackQuery, utils::html::bold,
};

pub async fn my_orders_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => return Ok(()),
    };

    let (msg, keyboard) = match api_client.get_user_orders(chat_id.0).await {
        Ok(orders) => {
            if orders.items.is_empty() {
                (
                    "У вас пока нет заказов.".to_string(),
                    back_to_main_menu_inline_keyboard(),
                )
            } else {
                (
                    format!("{}\n\n", bold("🧾 Ваши заказы:")),
                    my_orders_inline_keyboard(&orders.items),
                )
            }
        }
        Err(_) => (
            "Произошла ошибка при получении заказов. Попробуйте позже.".to_string(),
            back_to_main_menu_inline_keyboard(),
        ),
    };

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &crate::bot::utils::MsgBy::CallbackQuery(&q),
        &msg,
        None,
        keyboard,
    )
    .await?;

    Ok(())
}
