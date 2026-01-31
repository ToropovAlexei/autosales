use std::sync::Arc;

use crate::{
    api::backend_api::BackendApi,
    bot::{
        BotState, BotStep, MyDialogue,
        keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard,
        utils::{MsgBy, edit_msg},
    },
    errors::AppResult,
};
use teloxide::prelude::*;

pub async fn add_bot_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    bot_state: BotState,
) -> AppResult<()> {
    let percentage = api_client.get_settings().await?.referral_percentage;
    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
         &format!(
        "Вы можете создать свой собственный магазин-бот и получать <b>{}%</b> с каждой продажи!\n\n\
         Для этого:\n\
         1. Создайте нового бота через @BotFather в Telegram.\n\
         2. Получите у него токен (набор символов вида `123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11`).\n\
         3. Отправьте этот токен мне в следующем сообщении.\n\n\
         Я жду ваш токен.",
        percentage
        ),
        None,
        back_to_main_menu_inline_keyboard(),
    )
    .await?;

    dialogue
        .update(BotState {
            step: BotStep::WaitingForReferralBotToken,
            ..bot_state
        })
        .await?;

    Ok(())
}
