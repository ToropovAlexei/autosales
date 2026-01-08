use crate::{bot::BotUsername, errors::AppResult};
use teloxide::prelude::*;

#[allow(dead_code)]
pub async fn fallback_bot_msg(
    bot: Bot,
    chat_id: ChatId,
    fallback_bot_username: BotUsername,
) -> AppResult<()> {
    let new_text: String = format!("🤖 Наш резервный бот: @{}", fallback_bot_username.0);
    let chat = bot.get_chat(chat_id).await?;
    if let Some(pinned) = chat.pinned_message
        && let Some(text) = &pinned.text()
        && text == &new_text
    {
        return Ok(());
    }

    if let Err(err) = bot.unpin_all_chat_messages(chat_id).await {
        tracing::warn!("Failed to unpin all messages: {:?}", err);
    }

    let sent = bot.send_message(chat_id, new_text).await?;

    if let Err(err) = bot
        .pin_chat_message(chat_id, sent.id)
        .disable_notification(true)
        .await
    {
        tracing::warn!("Failed to pin message: {:?}", err);
    }

    Ok(())
}
