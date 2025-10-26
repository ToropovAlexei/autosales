use std::sync::Arc;

use teloxide::{
    Bot,
    dispatching::dialogue::GetChatId,
    prelude::{Request, Requester},
    types::{CallbackQuery, MaybeInaccessibleMessage, ParseMode},
    utils::html::bold,
};

use crate::{
    api::backend_api::BackendApi,
    bot::{MyDialogue, keyboards::balance_menu::balance_menu_inline_keyboard},
    errors::AppResult,
};
use teloxide::payloads::EditMessageTextSetters;

pub async fn balance_handler(
    bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    username: String,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => return Ok(()),
    };
    let message_id = match &q.message {
        Some(MaybeInaccessibleMessage::Regular(msg)) => msg.id,
        Some(MaybeInaccessibleMessage::Inaccessible(_)) => return Ok(()),
        None => return Ok(()),
    };
    match api_client.get_user_balance(chat_id.0, &username).await {
        Ok(balance) => {
            bot.edit_message_text(
                chat_id,
                message_id,
                format!("💳 Ваш текущий баланс: {} ₽", bold(&balance.to_string())),
            )
            .reply_markup(balance_menu_inline_keyboard())
            .parse_mode(ParseMode::Html)
            .send()
            .await?;
            Ok(())
        }
        Err(err) => {
            tracing::error!("Error getting balance: {err}");
            bot.edit_message_text(
                chat_id,
                message_id,
                "Ошибка получения баланса. Попробуйте позже.",
            )
            .send()
            .await?;
            Ok(())
        }
    }
}
