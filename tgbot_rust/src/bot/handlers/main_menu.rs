use std::sync::Arc;

use teloxide::{Bot, types::CallbackQuery};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        MyDialogue,
        keyboards::main_menu::main_menu_inline_keyboard,
        utils::{MsgBy, edit_msg},
    },
    errors::AppResult,
};

pub async fn main_menu_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let is_referral_program_enabled = api_client.is_referral_program_enabled().await;

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        "Главное меню",
        None,
        main_menu_inline_keyboard(is_referral_program_enabled),
    )
    .await?;

    Ok(())
}
