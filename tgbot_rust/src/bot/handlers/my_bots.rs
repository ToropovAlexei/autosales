use std::sync::Arc;
use teloxide::prelude::*;
use crate::{
    bot::MyDialogue,
    errors::AppResult,
    AppState,
};

pub async fn my_bots_handler(
    bot: Bot,
    dialogue: MyDialogue,
    app_state: AppState,
) -> AppResult<()> {
    bot.send_message(dialogue.chat_id(), "Please send me the token of your bot.")
        .await?;
    dialogue.update(crate::bot::BotState::WaitingForReferralBotToken).await?;
    Ok(())
}

pub async fn referral_bot_token_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    app_state: AppState,
) -> AppResult<()> {
    let token = msg.text().unwrap_or_default();
    let telegram_id = msg.chat.id.0;
    match app_state.api.create_referral_bot(telegram_id, token).await {
        Ok(_) => {
            bot.send_message(dialogue.chat_id(), "Your referral bot has been created successfully!").await?;
        }
        Err(e) => {
            bot.send_message(dialogue.chat_id(), format!("Failed to create referral bot: {}", e)).await?;
        }
    }
    dialogue.update(crate::bot::BotState::MainMenu).await?;
    Ok(())
}
