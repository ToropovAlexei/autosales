use std::sync::Arc;

use crate::bot::keyboards::captcha::captcha_keyboard_inline;
use crate::bot::keyboards::main_menu::main_menu_inline_keyboard;
use crate::bot::keyboards::main_menu_reply::main_menu_reply_keyboard;
use crate::bot::utils::{MessageImage, MsgBy, edit_msg, send_msg};
use crate::bot::{BotState, BotStep, generate_captcha_and_options};
use crate::{api::backend_api::BackendApi, bot::MyDialogue, errors::AppResult};
use chrono::Utc;
use shared_dtos::customer::UpdateCustomerBotRequest;
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::types::{InlineKeyboardMarkup, Message, MessageId, ReplyMarkup};

pub async fn start_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
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

    if let Some(blocked_until) = user.blocked_until
        && blocked_until > Utc::now()
    {
        let minutes = (blocked_until - Utc::now()).num_minutes();
        let hours = (minutes as f64 / 60.0).ceil() as i64;
        let hours_str = match hours {
            1 => "час",
            2..=4 => "часа",
            _ => "часов",
        };
        edit_msg(
            &api_client,
            &dialogue,
            &bot,
            &MsgBy::Message(&msg),
            &format!("Ваш аккаунт заблокирован на {hours} {hours_str}"),
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
                ..dialogue.get().await?.unwrap_or_default()
            })
            .await?;
        return Ok(());
    }

    dialogue
        .update(BotState {
            step: BotStep::MainMenu,
            ..dialogue.get().await?.unwrap_or_default()
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

    let prev_state = dialogue.get().await?.unwrap_or_default();

    if let Some(prev_welcome_msg) = prev_state.last_bot_welcome_msg_id {
        let _ = bot
            .delete_message(user_id, MessageId(prev_welcome_msg as i32))
            .await;
    }

    // Send dummy message just to show reply keyboard
    let dummy_msg = send_msg(
        &api_client,
        &dialogue,
        &bot,
        &welcome_msg,
        welcome_msg_img_id,
        ReplyMarkup::Keyboard(main_menu_reply_keyboard()),
    )
    .await?;

    // Reset msg id to prevent dummy msg removal
    dialogue
        .update(BotState {
            last_bot_msg_id: prev_state.last_bot_msg_id,
            last_bot_welcome_msg_id: Some(dummy_msg.id.0 as i64),
            ..dialogue.get().await?.unwrap_or_default()
        })
        .await?;

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::Message(&msg),
        "Главное меню",
        None,
        main_menu_inline_keyboard(referral_program_enabled),
    )
    .await?;

    Ok(())
}
