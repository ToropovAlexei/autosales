use std::sync::Arc;

use crate::bot::keyboards::captcha::captcha_keyboard_inline;
use crate::bot::keyboards::main_menu::main_menu_inline_keyboard;
use crate::bot::{BotState, generate_captcha_and_options};
use crate::{api::backend_api::BackendApi, bot::MyDialogue, errors::AppResult};
use teloxide::Bot;
use teloxide::payloads::{SendMessageSetters, SendPhotoSetters};
use teloxide::prelude::Request;
use teloxide::prelude::Requester;
use teloxide::types::{InputFile, Message, ParseMode};

pub async fn start_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let user_id = msg.chat.id;
    let user = match api_client.register_user(user_id.0).await {
        Ok(res) => res,
        Err(err) => {
            tracing::error!("Error getting user: {user_id}, {err}");
            bot.send_message(
                msg.chat.id,
                "Произошла непредвиденная ошибка. Попробуйте позже.",
            )
            .send()
            .await?;
            return Ok(());
        }
    };

    if user.is_blocked {
        tracing::info!("User is blocked: {user_id}");
        bot.send_message(msg.chat.id, "Ваш аккаунт заблокирован")
            .send()
            .await?;
        return Ok(());
    }

    if !user.has_passed_captcha {
        let (png_bytes, captcha_text, options) =
            match generate_captcha_and_options(api_client).await {
                Ok((i, a, o)) => (i, a, o),
                Err(e) => {
                    tracing::error!("Error generating captcha: {e}");
                    bot.send_message(msg.chat.id, "Что-то пошло не так. Попробуйте ещё раз")
                        .send()
                        .await?;
                    return Ok(());
                }
            };

        let keyboard = captcha_keyboard_inline(&options);

        bot.send_photo(msg.chat.id, InputFile::memory(png_bytes))
            .caption("Пожалуйста, решите капчу, чтобы продолжить:")
            .reply_markup(keyboard)
            .await?;

        dialogue
            .update(BotState::WaitingForCaptcha {
                correct_answer: captcha_text,
            })
            .await?;
        return Ok(());
    }

    dialogue.update(BotState::MainMenu).await?;

    let referral_program_enabled = api_client.is_referral_program_enabled().await;
    // TODO Не has_passed_captcha, а юзер пришел еще раз
    let welcome_message = if !user.has_passed_captcha {
        match api_client.get_new_user_welcome_msg().await {
            Some(m) => m.replace(
                "{username}",
                msg.from
                    .map(|user| user.full_name())
                    .unwrap_or_default()
                    .as_str(),
            ),
            None => {
                tracing::error!("No new user welcome message found");
                "Что-то пошло не так. Попробуйте ещё раз".to_string()
            }
        }
    } else {
        match api_client.get_returning_user_welcome_msg().await {
            Some(m) => m.replace(
                "{username}",
                msg.from
                    .map(|user| user.full_name())
                    .unwrap_or_default()
                    .as_str(),
            ),
            None => {
                tracing::error!("No returning user welcome message found");
                "Что-то пошло не так. Попробуйте ещё раз".to_string()
            }
        }
    };

    bot.send_message(msg.chat.id, welcome_message)
        .parse_mode(ParseMode::Html)
        .reply_markup(main_menu_inline_keyboard(referral_program_enabled))
        .await?;

    Ok(())
}
