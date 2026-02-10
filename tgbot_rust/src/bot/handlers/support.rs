use std::sync::Arc;

use teloxide::{
    Bot,
    types::{CallbackQuery, Message},
};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        MyDialogue,
        keyboards::main_menu::main_menu_inline_keyboard,
        utils::{MessageImage, MsgBy, edit_msg},
    },
    errors::AppResult,
};

pub async fn support_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    support_handler_impl(bot, dialogue, MsgBy::CallbackQuery(&q), api_client).await
}

pub async fn support_handler_msg(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    support_handler_impl(bot, dialogue, MsgBy::Message(&msg), api_client).await
}

async fn support_handler_impl(
    bot: Bot,
    dialogue: MyDialogue,
    msg_by: MsgBy<'_>,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let settings = api_client.get_settings().await?;
    let support_img = settings
        .bot_messages_support_image_id
        .map(MessageImage::Uuid);

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &msg_by,
        &settings.bot_messages_support,
        support_img,
        main_menu_inline_keyboard(settings.referral_program_enabled),
    )
    .await?;

    Ok(())
}
