use std::sync::Arc;

use crate::{
    api::backend_api::BackendApi,
    bot::{
        BotState, MyDialogue,
        keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard,
        utils::{MsgBy, edit_msg},
    },
    errors::AppResult,
};
use teloxide::prelude::*;

pub async fn delete_bot_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    _bot_state: BotState,
    id: i64,
) -> AppResult<()> {
    api_client.delete_bot(id).await?;
    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        "Бот удален.",
        None,
        back_to_main_menu_inline_keyboard(),
    )
    .await?;

    Ok(())
}
