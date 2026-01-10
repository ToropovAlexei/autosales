use std::sync::Arc;

use teloxide::{Bot, dispatching::dialogue::GetChatId, types::CallbackQuery, utils::html::bold};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        MyDialogue,
        keyboards::balance_menu::balance_menu_inline_keyboard,
        utils::{MsgBy, edit_msg},
    },
    errors::{AppError, AppResult},
};

pub async fn balance_handler(
    bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let telegram_id = q
        .chat_id()
        .map(|c| c.0)
        .ok_or(AppError::InternalServerError(
            "Failed to get telegram id".to_string(),
        ))?;
    let text = match api_client.get_user(telegram_id).await {
        Ok(customer) => format!(
            "💳 Ваш текущий баланс: {} ₽",
            bold(&customer.balance.to_string())
        ),
        Err(err) => {
            tracing::error!("Error getting balance: {err}");
            "Ошибка получения баланса. Попробуйте позже.".to_string()
        }
    };
    edit_msg(
        &api_client,
        &bot,
        &MsgBy::CallbackQuery(q),
        &text,
        None,
        balance_menu_inline_keyboard(),
    )
    .await?;
    Ok(())
}
