use std::sync::Arc;

use crate::{
    api::backend_api::BackendApi,
    bot::{
        MyDialogue,
        keyboards::my_bots_menu::my_bots_inline_keyboard,
        utils::{MsgBy, edit_msg},
    },
    errors::AppResult,
};
use teloxide::{dispatching::dialogue::GetChatId, prelude::*};

pub async fn referral_program_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let telegram_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => return Ok(()),
    };
    let bots = api_client.get_customer_bots(telegram_id.0).await?;
    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        "Управление вашими реферальными ботами:",
        None,
        my_bots_inline_keyboard(&bots.items),
    )
    .await?;

    Ok(())
}
