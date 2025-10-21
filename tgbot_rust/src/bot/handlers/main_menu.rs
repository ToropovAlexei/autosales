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

pub async fn main_menu_handler(
    bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    _username: String,
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
    let is_referral_program_enabled = api_client.is_referral_program_enabled().await;
    bot.edit_message_text(chat_id, message_id, "Главное меню")
        .reply_markup(main_menu_inline_keyboard(is_referral_program_enabled))
        .parse_mode(ParseMode::Html)
        .send()
        .await?;
    Ok(())
}
