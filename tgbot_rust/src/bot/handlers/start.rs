use std::sync::Arc;

use crate::bot::keyboards::captcha::captcha_keyboard_inline;
use crate::bot::keyboards::main_menu::main_menu_inline_keyboard;
use crate::bot::utils::{MessageImage, MsgBy, edit_msg};
use crate::bot::{BotState, BotStep, generate_captcha_and_options};
use crate::{api::backend_api::BackendApi, bot::MyDialogue, errors::AppResult};
use shared_dtos::customer::UpdateCustomerBotRequest;
use teloxide::Bot;
use teloxide::types::{InlineKeyboardMarkup, Message};

pub async fn start_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let state_data = dialogue.get().await?.unwrap_or_default();
    let user_id = msg.chat.id;
    let user = match api_client.ensure_user(user_id.0).await {
        Ok(res) => res,
        Err(err) => {
            tracing::error!("Error getting user: {user_id}, {err}");
            edit_msg(
                &api_client,
                &dialogue,
                &bot,
                &MsgBy::Message(&msg),
                "Произошла непредвиденная ошибка. Попробуйте позже.",
                None,
                InlineKeyboardMarkup::default(),
            )
            .await?;
            return Ok(());
        }
    };

    if user.is_blocked {
        tracing::info!("User is blocked: {user_id}");
        edit_msg(
            &api_client,
            &dialogue,
            &bot,
            &MsgBy::Message(&msg),
            "Ваш аккаунт заблокирован",
            None,
            InlineKeyboardMarkup::default(),
        )
        .await?;
        return Ok(());
    }

    if user.bot_is_blocked_by_user {
        api_client
            .update_customer(
                user.telegram_id,
                &UpdateCustomerBotRequest {
                    bot_is_blocked_by_user: Some(false),
                    ..Default::default()
                },
            )
            .await?;
    }

    if !user.has_passed_captcha {
        let (png_bytes, captcha_text, options) =
            match generate_captcha_and_options(&api_client).await {
                Ok((i, a, o)) => (i, a, o),
                Err(e) => {
                    tracing::error!("Error generating captcha: {e}");
                    edit_msg(
                        &api_client,
                        &dialogue,
                        &bot,
                        &MsgBy::Message(&msg),
                        "Что-то пошло не так. Попробуйте ещё раз",
                        None,
                        InlineKeyboardMarkup::default(),
                    )
                    .await?;
                    return Ok(());
                }
            };

        let keyboard = captcha_keyboard_inline(&options);

        edit_msg(
            &api_client,
            &dialogue,
            &bot,
            &MsgBy::Message(&msg),
            "Пожалуйста, решите капчу, чтобы продолжить:",
            Some(MessageImage::Bytes(png_bytes)),
            keyboard,
        )
        .await?;

        dialogue
            .update(BotState {
                step: BotStep::WaitingForCaptcha {
                    correct_answer: captcha_text,
                },
                ..state_data
            })
            .await?;
        return Ok(());
    }

    dialogue
        .update(BotState {
            step: BotStep::MainMenu,
            ..state_data
        })
        .await?;

    let referral_program_enabled = api_client.is_referral_program_enabled().await;
    let settings = api_client.get_settings().await?;
    // TODO Не has_passed_captcha, а юзер пришел еще раз
    let (welcome_msg, image_id) = if !user.has_passed_captcha {
        (
            settings.bot_messages_new_user_welcome,
            settings.bot_messages_new_user_welcome_image_id,
        )
    } else {
        (
            settings.bot_messages_returning_user_welcome,
            settings.bot_messages_returning_user_welcome_image_id,
        )
    };
    let welcome_msg = welcome_msg.replace(
        "{username}",
        msg.clone()
            .from
            .map(|user| user.full_name())
            .unwrap_or_default()
            .as_str(),
    );
    let welcome_msg_img_id = image_id.map(MessageImage::Uuid);

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::Message(&msg),
        &welcome_msg,
        welcome_msg_img_id,
        main_menu_inline_keyboard(referral_program_enabled),
    )
    .await?;

    Ok(())
}
