use std::sync::Arc;

use teloxide::{Bot, types::CallbackQuery, types::Message};

use crate::bot::handlers::balance::balance_handler_msg;
use crate::bot::handlers::catalog::catalog_handler_msg;
use crate::bot::handlers::support::support_handler_msg;
use crate::bot::{BotState, BotStep};
use crate::{
    api::backend_api::BackendApi,
    bot::{
        MyDialogue,
        keyboards::main_menu::main_menu_inline_keyboard,
        keyboards::main_menu_reply::MainMenuReplyAction,
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

pub async fn main_menu_text_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let action = msg.text().and_then(MainMenuReplyAction::from_text);
    let prev_state = dialogue.get_or_default().await.unwrap_or_default();

    match action {
        Some(MainMenuReplyAction::Catalog) => {
            dialogue
                .update(BotState {
                    step: BotStep::Category { category_id: None },
                    ..prev_state
                })
                .await?;
            catalog_handler_msg(bot, dialogue, msg, api_client, None).await?;
        }
        Some(MainMenuReplyAction::Balance) => {
            dialogue
                .update(BotState {
                    step: BotStep::Balance,
                    ..prev_state
                })
                .await?;
            balance_handler_msg(bot, dialogue, msg, api_client).await?;
        }
        Some(MainMenuReplyAction::Support) => {
            dialogue
                .update(BotState {
                    step: BotStep::Support,
                    ..prev_state
                })
                .await?;
            support_handler_msg(bot, dialogue, msg, api_client).await?;
        }
        None => {}
    }

    Ok(())
}
