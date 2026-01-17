use std::sync::Arc;

use teloxide::{Bot, types::CallbackQuery};

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
    let settings = api_client.get_settings().await?;
    let support_img = settings
        .bot_messages_support_image_id
        .map(MessageImage::Uuid);

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        &settings.bot_messages_support,
        support_img,
        main_menu_inline_keyboard(settings.referral_program_enabled),
    )
    .await?;

    Ok(())
}
