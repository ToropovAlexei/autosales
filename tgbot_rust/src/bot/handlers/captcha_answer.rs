use std::sync::Arc;

use crate::api::captcha_api::CaptchaApi;
use crate::bot::keyboards::captcha::captcha_keyboard_inline;
use crate::bot::keyboards::main_menu::main_menu_inline_keyboard;
use crate::bot::{BotState, CallbackData, generate_captcha_and_options};
use crate::{api::backend_api::BackendApi, bot::MyDialogue, errors::AppResult};
use teloxide::Bot;
use teloxide::payloads::EditMessageMediaSetters;
use teloxide::payloads::{AnswerCallbackQuerySetters, SendMessageSetters};
use teloxide::prelude::Requester;
use teloxide::types::{CallbackQuery, InputFile, InputMedia, InputMediaPhoto, ParseMode};

pub async fn captcha_answer_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    _username: String,
    api_client: Arc<BackendApi>,
    captcha_api_client: Arc<CaptchaApi>,
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

            let referral_program_enabled = api_client.is_referral_program_enabled().await;
            let welcome_msg = match api_client.get_new_user_welcome_msg().await {
                Some(msg) => msg.replace("{username}", &q.from.full_name()),
                None => {
                    tracing::error!("No welcome message found");
                    return Ok(());
                }
            };

            bot.send_message(msg.chat().id, welcome_msg)
                .parse_mode(ParseMode::Html)
                .reply_markup(main_menu_inline_keyboard(referral_program_enabled))
                .await?;
        }

        dialogue.exit().await?;
    } else {
        bot.answer_callback_query(q.id.clone())
            .text("Неверный ответ, попробуйте еще раз.")
            .show_alert(true)
            .await?;

        let (captcha_image, new_correct_answer, options) =
            match generate_captcha_and_options(captcha_api_client, 6, 12).await {
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

        if let Some(msg) = q.message {
            bot.edit_message_media(
                msg.chat().id,
                msg.id(),
                InputMedia::Photo(InputMediaPhoto::new(InputFile::memory(captcha_image))),
            )
            .reply_markup(captcha_keyboard_inline(&options))
            .await?;
        }
    }

    Ok(())
}
