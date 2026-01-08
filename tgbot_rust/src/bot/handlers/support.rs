use std::sync::Arc;

use teloxide::{
    Bot,
    payloads::EditMessageTextSetters,
    prelude::Request,
    types::{CallbackQuery, MaybeInaccessibleMessage, ParseMode},
};

use crate::{
    api::backend_api::BackendApi,
    bot::{MyDialogue, keyboards::main_menu::main_menu_inline_keyboard},
    errors::AppResult,
};
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::Requester;

pub async fn support_handler(
    bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => {
            tracing::error!("No chat id found");
            return Ok(());
        }
    };
    let message_id = match &q.message {
        Some(MaybeInaccessibleMessage::Regular(msg)) => msg.id,
        Some(MaybeInaccessibleMessage::Inaccessible(_)) => {
            tracing::error!("Inaccessible message found");
            return Ok(());
        }
        None => {
            tracing::error!("No message found");
            return Ok(());
        }
    };
    let support_msg = match api_client.get_support_msg().await {
        Some(msg) => msg,
        None => {
            tracing::error!("No support message found");
            "Что-то пошло не так. Попробуйте ещё раз".to_string()
        }
    };
    let is_referral_program_enabled = api_client.is_referral_program_enabled().await;
    bot.edit_message_text(chat_id, message_id, support_msg)
        .reply_markup(main_menu_inline_keyboard(is_referral_program_enabled))
        .parse_mode(ParseMode::Html)
        .send()
        .await?;
    Ok(())
}
