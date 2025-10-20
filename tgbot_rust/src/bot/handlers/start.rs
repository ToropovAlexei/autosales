use std::sync::Arc;

use crate::bot::BotState;
use crate::bot::keyboards::captcha::captcha_keyboard_inline;
use crate::utils::generate_captcha_and_options;
use crate::{
    api::backend_api::BackendApi, bot::MyDialogue, errors::AppResult, models::user::BotUser,
};
use teloxide::Bot;
use teloxide::payloads::SendPhotoSetters;
use teloxide::prelude::Request;
use teloxide::prelude::Requester;
use teloxide::types::{InputFile, Message};

pub async fn start_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    username: String,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let user_id = msg.chat.id;
    let user = match ensure_user(api_client, user_id.0, &username).await {
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
        bot.send_message(msg.chat.id, "Ваш аккаунт заблокирован")
            .send()
            .await?;
        return Ok(());
    }

    if !user.has_passed_captcha {
        let (png_bytes, captcha_text, options) = match generate_captcha_and_options(6, 12) {
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

    bot.send_message(msg.chat.id, "Дароу").send().await?;

    Ok(())
}

async fn ensure_user(
    api_client: Arc<BackendApi>,
    user_id: i64,
    bot_username: &str,
) -> AppResult<BotUser> {
    if let Ok(user) = api_client.get_user(user_id, bot_username).await {
        Ok(user)
    } else {
        api_client.register_user(user_id, bot_username).await
    }
}
