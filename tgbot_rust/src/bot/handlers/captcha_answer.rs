use std::sync::Arc;

use crate::api::captcha_api::CaptchaApi;
use crate::bot::keyboards::captcha::captcha_keyboard_inline;
use crate::bot::keyboards::main_menu::main_menu_inline_keyboard;
use crate::bot::{BotState, generate_captcha_and_options};
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
    let state = dialogue.get().await?;
    let correct_answer = match &state {
        Some(BotState::WaitingForCaptcha { correct_answer }) => correct_answer.clone(),
        _ => return Ok(()),
    };

    let data = match q.data.as_deref() {
        Some(d) => d,
        None => {
            tracing::error!("No callback data");
            bot.answer_callback_query(q.id)
                .text("Что-то пошло не так. Попробуйте ещё раз")
                .await?;
            return Ok(());
        }
    };

    if !data.starts_with("captcha_") {
        tracing::error!("Invalid callback data: {data}");
        bot.answer_callback_query(q.id)
            .text("Что-то пошло не так. Попробуйте ещё раз")
            .await?;
        return Ok(());
    }

    let answer = data.trim_start_matches("captcha_");

    if answer.to_string() == correct_answer {
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
            // Тут можно вызвать функцию для обновления pinned message и отправки приветствия
            // update_pinned_message(&bot, &msg).await?;
            // ...

            let settings = api_client.get_settings().await.unwrap_or_default();
            let referral_program_enabled = settings
                .get("referral_program_enabled")
                .as_ref()
                .map(|v| {
                    v.as_bool().unwrap_or_else(|| {
                        v.as_str()
                            .map(|s| s.eq_ignore_ascii_case("true"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            bot.send_message(
                msg.chat().id,
                format!(
                    "Добро пожаловать, {}!\n\n\
                    Я - ваш личный помощник для покупок. Здесь вы можете:\n\
                    - 🛍️ Смотреть каталог товаров\n\
                    - 💰 Пополнять баланс\n\
                    - 💳 Проверять свой счет\n\n\
                    Выберите действие в меню ниже:",
                    q.from.full_name(),
                ),
            )
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
