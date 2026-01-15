use std::sync::Arc;

use crate::bot::keyboards::captcha::captcha_keyboard_inline;
use crate::bot::keyboards::main_menu::main_menu_inline_keyboard;
use crate::bot::utils::{MessageImage, MsgBy, edit_msg};
use crate::bot::{BotState, CallbackData, generate_captcha_and_options};
use crate::{api::backend_api::BackendApi, bot::MyDialogue, errors::AppResult};
use teloxide::Bot;
use teloxide::payloads::AnswerCallbackQuerySetters;
use teloxide::prelude::Requester;
use teloxide::types::CallbackQuery;

pub async fn captcha_answer_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let state_data = dialogue.get().await?.unwrap_or_default();
    let correct_answer = match &state_data {
        BotState::WaitingForCaptcha { correct_answer } => correct_answer.clone(),
        _ => return Ok(()),
    };

    let user_answer = match CallbackData::from_query(&q) {
        Some(d) => match d {
            CallbackData::AnswerCaptcha { answer } => answer,
            _ => {
                tracing::error!("Invalid callback data");
                bot.answer_callback_query(q.id)
                    .text("Что-то пошло не так. Попробуйте ещё раз")
                    .await?;
                return Ok(());
            }
        },
        None => {
            tracing::error!("No callback data");
            bot.answer_callback_query(q.id)
                .text("Что-то пошло не так. Попробуйте ещё раз")
                .await?;
            return Ok(());
        }
    };

    let settings = api_client.get_settings().await?;

    if user_answer == correct_answer {
        if let Err(e) = api_client.confirm_user_captcha(dialogue.chat_id().0).await {
            tracing::error!("Error confirming user captcha: {e}");
            bot.answer_callback_query(q.id)
                .text("Что-то пошло не так. Попробуйте ещё раз")
                .show_alert(true)
                .await?;
            return Ok(());
        }

        if let Some(msg) = q.message.clone() {
            bot.delete_message(msg.chat().id, msg.id()).await.ok();
            let welcome_msg_img = settings
                .bot_messages_new_user_welcome_image_id
                .map(MessageImage::Uuid);

            edit_msg(
                &api_client,
                &bot,
                &MsgBy::CallbackQuery(&q),
                &settings
                    .bot_messages_new_user_welcome
                    .replace("{username}", &&q.from.first_name),
                welcome_msg_img,
                main_menu_inline_keyboard(settings.referral_program_enabled),
            )
            .await?;
        }

        dialogue.exit().await?;
    } else {
        bot.answer_callback_query(q.id.clone())
            .text("Неверный ответ, попробуйте еще раз.")
            .show_alert(true)
            .await?;

        let (captcha_image, new_correct_answer, options) =
            match generate_captcha_and_options(&api_client).await {
                Ok((i, a, o)) => (i, a, o),
                Err(e) => {
                    tracing::error!("Error generating captcha: {e}");
                    bot.answer_callback_query(q.id)
                        .text("Что-то пошло не так. Попробуйте ещё раз")
                        .show_alert(true)
                        .await?;
                    return Ok(());
                }
            };
        dialogue
            .update(BotState::WaitingForCaptcha {
                correct_answer: new_correct_answer,
            })
            .await?;

        edit_msg(
            &api_client,
            &bot,
            &MsgBy::CallbackQuery(&q),
            "Пожалуйста, решите капчу, чтобы продолжить:",
            Some(MessageImage::Bytes(captcha_image)),
            captcha_keyboard_inline(&options),
        )
        .await?;
    }

    Ok(())
}
