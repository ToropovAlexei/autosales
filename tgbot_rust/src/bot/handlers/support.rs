use std::sync::Arc;

use teloxide::{
    Bot,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        CallbackData, MyDialogue,
        utils::{MessageImage, MsgBy, edit_msg, support_operator_buttons},
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
    let support_rows = support_operator_buttons(&settings.bot_store_support_operators);
    let keyboard = InlineKeyboardMarkup::new(
        support_rows
            .into_iter()
            .chain(std::iter::once(vec![InlineKeyboardButton::callback(
                "⬅️ Назад",
                CallbackData::ToMainMenu,
            )]))
            .collect::<Vec<_>>(),
    );

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &msg_by,
        &settings.bot_messages_support,
        support_img,
        keyboard,
    )
    .await?;

    Ok(())
}
