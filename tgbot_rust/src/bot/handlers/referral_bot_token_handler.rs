use std::sync::Arc;

use crate::bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard;
use crate::bot::utils::{MsgBy, edit_msg};
use crate::bot::{BotState, BotStep};
use crate::{api::backend_api::BackendApi, bot::MyDialogue, errors::AppResult};
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::types::{Message, MessageId};

pub async fn referral_bot_token_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    api_client: Arc<BackendApi>,
    bot_state: BotState,
) -> AppResult<()> {
    let token = if let Some(token) = msg.text() {
        token
    } else {
        edit_msg(
            &api_client,
            &dialogue,
            &bot,
            &MsgBy::Message(&msg),
            "Пожалуйста, пришлите токен бота.",
            None,
            back_to_main_menu_inline_keyboard(),
        )
        .await?;

        return Ok(());
    };

    let _ = bot.delete_message(msg.chat.id, msg.id).await;
    if let Some(msg_id) = bot_state.last_bot_msg_id {
        let _ = bot
            .delete_message(msg.chat.id, MessageId(msg_id as i32))
            .await;
    }

    if !validate_token(token) {
        edit_msg(
            &api_client,
            &dialogue,
            &bot,
            &MsgBy::Message(&msg),
            "Пожалуйста, пришлите корректный токен бота.",
            None,
            back_to_main_menu_inline_keyboard(),
        )
        .await?;

        return Ok(());
    }

    match api_client.create_referral_bot(msg.chat.id.0, token).await {
        Ok(_) => {
            edit_msg(
                &api_client,
                &dialogue,
                &bot,
                &MsgBy::Message(&msg),
                "🎉 Поздравляем! Ваш реферальный бот успешно создан.",
                None,
                back_to_main_menu_inline_keyboard(),
            )
            .await?;
            dialogue
                .update(BotState {
                    step: BotStep::MainMenu,
                    ..bot_state
                })
                .await?;
        }
        Err(e) => {
            let answer = {
                match e.to_string().contains("Unique violation") {
                    true => "Такой бот уже есть.",
                    false => "Что-то пошло не так, попробуйте позже.",
                }
            };
            edit_msg(
                &api_client,
                &dialogue,
                &bot,
                &MsgBy::Message(&msg),
                answer, // TODO Handle errors
                None,
                back_to_main_menu_inline_keyboard(),
            )
            .await?;
            return Ok(());
        }
    };

    Ok(())
}

fn validate_token(token: &str) -> bool {
    if token.len() < 44 {
        return false;
    }
    if token.len() > 60 {
        return false;
    }
    true
}
