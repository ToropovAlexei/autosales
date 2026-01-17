use crate::{
    AppState,
    bot::{BotState, BotStep, MyDialogue},
    errors::AppResult,
};
use teloxide::prelude::*;

pub async fn my_bots_handler(
    bot: Bot,
    dialogue: MyDialogue,
    _app_state: AppState,
) -> AppResult<()> {
    let state_data = dialogue.get().await?.unwrap_or_default();
    bot.send_message(dialogue.chat_id(), "Please send me the token of your bot.")
        .await?;
    dialogue
        .update(BotState {
            step: BotStep::WaitingForReferralBotToken,
            ..state_data
        })
        .await?;
    Ok(())
}

pub async fn referral_bot_token_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    app_state: AppState,
) -> AppResult<()> {
    let state_data = dialogue.get().await?.unwrap_or_default();
    let token = msg.text().unwrap_or_default();
    let telegram_id = msg.chat.id.0;
    match app_state.api.create_referral_bot(telegram_id, token).await {
        Ok(_) => {
            bot.send_message(
                dialogue.chat_id(),
                "Your referral bot has been created successfully!",
            )
            .await?;
        }
        Err(e) => {
            bot.send_message(
                dialogue.chat_id(),
                format!("Failed to create referral bot: {}", e),
            )
            .await?;
        }
    }
    dialogue
        .update(BotState {
            step: BotStep::MainMenu,
            ..state_data
        })
        .await?;
    Ok(())
}
