use std::sync::Arc;

use bytes::Bytes;
use shared_dtos::invoice::PaymentDetails;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::payloads::{
    EditMessageMediaSetters, EditMessageTextSetters, SendMessageSetters, SendPhotoSetters,
};
use teloxide::prelude::{Request, Requester};
use teloxide::types::{
    CallbackQuery, ChatId, InlineKeyboardMarkup, InputFile, InputMedia, InputMediaPhoto,
    MaybeInaccessibleMessage, Message, MessageId, ParseMode, ReplyMarkup,
};
use teloxide::utils::html::escape;
use uuid::Uuid;

use crate::api::backend_api::BackendApi;
use crate::bot::{BotState, InvoiceData, MyDialogue};
use crate::errors::{AppError, AppResult};

use teloxide::Bot;

pub enum MsgBy<'a> {
    Message(&'a Message),
    CallbackQuery(&'a CallbackQuery),
}

#[derive(Debug)]
pub enum MessageImage {
    Uuid(Uuid),
    Bytes(Bytes),
}

pub fn invoice_troubles_paragraph(
    amount: impl std::fmt::Display,
    rounded_minutes_left: i64,
) -> String {
    format!(
        "<b>Вы недавно пытались пополнить баланс на {amount} ₽.</b>\nВозникли ли у вас какие-либо проблемы с оплатой?\n\
         <u>У вас осталось {rounded_minutes_left} минут на оплату</u>"
    )
}

pub fn build_invoice_payment_text(
    invoice_data: &InvoiceData,
    requested_amount: i64,
    troubles_minutes_left: Option<i64>,
) -> String {
    let mut text = match &invoice_data.details {
        None => "Не удалось получить реквизиты для оплаты. Попробуйте другой способ.".to_string(),
        Some(details) => match details {
            PaymentDetails::Mock { .. } => format!(
                "✅ Ваш счет на {} ₽ создан.\n\nНажмите на кнопку ниже, чтобы перейти к оплате.",
                requested_amount
            ),
            PaymentDetails::PlatformCard {
                bank_name,
                account_name,
                card_number,
                amount,
            } => {
                let bank_name = escape(bank_name);
                let account_name = escape(account_name);
                let card_number = escape(card_number);
                let token = escape(&invoice_data.gateway_invoice_id);
                format!(
                    "Реквизиты для оплаты:\n\n\
                     <b>Банк:</b> {bank_name}\n\
                     <b>Номер карты:</b> <code>{card_number}</code>\n\
                     <b>Получатель:</b> {account_name}\n\
                     <b>Сумма:</b> <code>{amount}</code> ₽\n\n\
                     <b>Токен:</b> <code>{token}</code>\n\n\
                     <u>На оплату дается 30 минут!</u>\n\
                     В случае, если вы не оплатите в течении 30 минут, платеж не будет зачислен!\n\
                     <b>После оплаты ОБЯЗАТЕЛЬНО НАЖМИТЕ \"Оплатил\"</b>"
                )
            }
            PaymentDetails::PlatformSBP {
                bank_name,
                account_name,
                sbp_number,
                amount,
            } => {
                let bank_name = escape(bank_name);
                let account_name = escape(account_name);
                let sbp_number = escape(sbp_number);
                let token = escape(&invoice_data.gateway_invoice_id);
                format!(
                    "Реквизиты для оплаты:\n\n\
                     <b>Банк:</b> {bank_name}\n\
                     <b>Номер СБП:</b> <code>{sbp_number}</code>\n\
                     <b>Получатель:</b> {account_name}\n\
                     <b>Сумма:</b> <code>{amount} ₽</code>\n\n\
                     <b>Токен:</b> <code>{token}</code>\n\n\
                     <u>На оплату дается 30 минут!</u>\n\
                     В случае, если вы не оплатите в течении 30 минут, платеж не будет зачислен!\n\
                     <b>После оплаты ОБЯЗАТЕЛЬНО НАЖМИТЕ \"Оплатил\"</b>"
                )
            }
        },
    };

    if let Some(minutes_left) = troubles_minutes_left {
        text.push_str("\n\n");
        text.push_str(&invoice_troubles_paragraph(requested_amount, minutes_left));
    }

    text
}

/// Send message with or without photo
pub async fn send_msg(
    api_client: &Arc<BackendApi>,
    dialogue: &MyDialogue,
    bot: &Bot,
    text: &str,
    image: Option<MessageImage>,
    reply_keyboard: ReplyMarkup,
) -> AppResult<Message> {
    let image_bytes = match image {
        Some(MessageImage::Uuid(uuid)) => {
            let bytes = api_client.get_image_bytes(&uuid).await?;
            Some(bytes)
        }
        Some(MessageImage::Bytes(bytes)) => Some(bytes),
        None => None,
    };

    let msg = send_msg_impl(bot, dialogue.chat_id(), text, image_bytes, reply_keyboard).await?;

    let prev_state = dialogue.get_or_default().await.unwrap_or_default();
    dialogue
        .update(BotState {
            last_bot_msg_id: Some(msg.id.0 as i64),
            ..prev_state
        })
        .await?;

    Ok(msg)
}

/// Edit msg, if edit is failed, send new message
/// Automatically detect type of message
pub async fn edit_msg(
    api_client: &Arc<BackendApi>,
    dialogue: &MyDialogue,
    bot: &Bot,
    msg_by: &MsgBy<'_>,
    text: &str,
    image: Option<MessageImage>,
    reply_keyboard: InlineKeyboardMarkup,
) -> AppResult<Message> {
    let (chat_id, mut msg_id) = get_chat_and_msg_id(msg_by).ok_or(
        AppError::InternalServerError("Failed to get chat id".to_string()),
    )?;
    let prev_state = dialogue.get_or_default().await.unwrap_or_default();
    let mut has_photo = has_photo(msg_by);
    if let MsgBy::Message(msg) = msg_by {
        // Delete user message for cleanup
        let _ = bot.delete_message(chat_id, msg.id).await;
        let bot_msg_id = prev_state.last_bot_msg_id;
        if let Some(bot_msg_id) = bot_msg_id {
            // Delete bot message for cleanup
            let _ = bot
                .delete_message(chat_id, MessageId(bot_msg_id as i32))
                .await;
        }
        // Set msg_id to none, to prevent from trying to edit
        msg_id = None;
        has_photo = false;
    };
    let image_bytes = match image {
        Some(MessageImage::Uuid(uuid)) => {
            let bytes = api_client.get_image_bytes(&uuid).await?;
            Some(bytes)
        }
        Some(MessageImage::Bytes(bytes)) => Some(bytes),
        None => None,
    };
    let msg = edit_msg_impl(
        bot,
        chat_id,
        msg_id,
        has_photo,
        text,
        image_bytes,
        reply_keyboard,
    )
    .await?;

    dialogue
        .update(BotState {
            last_bot_msg_id: Some(msg.id.0 as i64),
            ..prev_state
        })
        .await?;
    Ok(msg)
}

async fn edit_msg_impl(
    bot: &Bot,
    chat_id: ChatId,
    msg_id: Option<MessageId>,
    has_photo: bool,
    text: &str,
    image_bytes: Option<Bytes>,
    reply_keyboard: InlineKeyboardMarkup,
) -> AppResult<Message> {
    // Msg id found, try to edit it
    if let Some(msg_id) = msg_id {
        // Telegram does not allow to change type of message
        let is_type_changed = image_bytes.is_some() != has_photo;
        if is_type_changed {
            // We can safely ignore error as it expected behavior
            let _ = bot.delete_message(chat_id, msg_id).await;
            return send_msg_impl(
                bot,
                chat_id,
                text,
                image_bytes,
                ReplyMarkup::InlineKeyboard(reply_keyboard),
            )
            .await;
        }
        return match image_bytes {
            Some(image_bytes) => {
                edit_media_msg(
                    bot,
                    chat_id,
                    Some(msg_id),
                    text,
                    image_bytes,
                    reply_keyboard,
                )
                .await
            }
            None => edit_text_msg(bot, chat_id, Some(msg_id), text, reply_keyboard).await,
        };
    }
    // If no msg id, just send new message
    send_msg_impl(
        bot,
        chat_id,
        text,
        image_bytes,
        ReplyMarkup::InlineKeyboard(reply_keyboard),
    )
    .await
}

/// Send message with or without photo
async fn send_msg_impl(
    bot: &Bot,
    chat_id: ChatId,
    text: &str,
    image_bytes: Option<Bytes>,
    reply_keyboard: ReplyMarkup,
) -> AppResult<Message> {
    match image_bytes {
        Some(image_bytes) => send_media_msg(bot, chat_id, text, image_bytes, reply_keyboard).await,
        None => send_text_msg(bot, chat_id, text, reply_keyboard).await,
    }
}

/// Check if message has photo
fn has_photo(msg_by: &MsgBy) -> bool {
    match msg_by {
        MsgBy::Message(msg) => msg.photo().is_some(),
        MsgBy::CallbackQuery(q) => match &q.message {
            Some(MaybeInaccessibleMessage::Regular(msg)) => msg.photo().is_some(),
            _ => false,
        },
    }
}

pub fn get_chat_and_msg_id(msg_by: &MsgBy) -> Option<(ChatId, Option<MessageId>)> {
    match msg_by {
        MsgBy::Message(msg) => Some((msg.chat.id, Some(msg.id))),
        MsgBy::CallbackQuery(q) => {
            let chat_id = q.chat_id()?;
            let message_id = match &q.message {
                Some(MaybeInaccessibleMessage::Regular(msg)) => Some(msg.id),
                Some(MaybeInaccessibleMessage::Inaccessible(_)) => None,
                None => None,
            };
            Some((chat_id, message_id))
        }
    }
}

/// Try edit message with photo. If edit is failed, send new message
async fn edit_media_msg(
    bot: &Bot,
    chat_id: ChatId,
    msg_id: Option<MessageId>,
    text: &str,
    image_bytes: Bytes,
    reply_keyboard: InlineKeyboardMarkup,
) -> AppResult<Message> {
    if let Some(msg_id) = msg_id {
        let input_media = InputMediaPhoto::new(InputFile::memory(image_bytes.clone()))
            .caption(text)
            .parse_mode(ParseMode::Html);
        if let Ok(msg) = bot
            .edit_message_media(chat_id, msg_id, InputMedia::Photo(input_media))
            .reply_markup(reply_keyboard.clone())
            .send()
            .await
        {
            return Ok(msg);
        }
    }
    send_media_msg(
        bot,
        chat_id,
        text,
        image_bytes,
        ReplyMarkup::InlineKeyboard(reply_keyboard),
    )
    .await
}

/// Try edit text message. If edit is failed, send new message
async fn edit_text_msg(
    bot: &Bot,
    chat_id: ChatId,
    msg_id: Option<MessageId>,
    text: &str,
    reply_keyboard: InlineKeyboardMarkup,
) -> AppResult<Message> {
    if let Some(msg_id) = msg_id
        && let Ok(msg) = bot
            .edit_message_text(chat_id, msg_id, text)
            .reply_markup(reply_keyboard.clone())
            .parse_mode(ParseMode::Html)
            .send()
            .await
    {
        return Ok(msg);
    }
    send_text_msg(
        bot,
        chat_id,
        text,
        ReplyMarkup::InlineKeyboard(reply_keyboard),
    )
    .await
}

/// Send message with photo
async fn send_media_msg(
    bot: &Bot,
    chat_id: ChatId,
    text: &str,
    image_bytes: Bytes,
    reply_keyboard: ReplyMarkup,
) -> AppResult<Message> {
    Ok(bot
        .send_photo(chat_id, InputFile::memory(image_bytes))
        .caption(text)
        .parse_mode(ParseMode::Html)
        .reply_markup(reply_keyboard)
        .send()
        .await?)
}

/// Send text message
async fn send_text_msg(
    bot: &Bot,
    chat_id: ChatId,
    text: &str,
    reply_keyboard: ReplyMarkup,
) -> AppResult<Message> {
    Ok(bot
        .send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(reply_keyboard)
        .send()
        .await?)
}
