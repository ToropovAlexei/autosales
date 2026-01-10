use std::sync::Arc;

use bytes::Bytes;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::payloads::{
    EditMessageMediaSetters, EditMessageTextSetters, SendMessageSetters, SendPhotoSetters,
};
use teloxide::prelude::{Request, Requester};
use teloxide::types::{
    CallbackQuery, ChatId, InlineKeyboardMarkup, InputFile, InputMedia, InputMediaPhoto,
    MaybeInaccessibleMessage, Message, MessageId, ParseMode,
};
use uuid::Uuid;

use crate::api::backend_api::BackendApi;
use crate::errors::{AppError, AppResult};

use teloxide::Bot;

/// Edit msg, if edit is failed, send new message
/// Automatically detect type of message
pub async fn edit_msg(
    api_client: &Arc<BackendApi>,
    bot: Bot,
    msg: Option<&Message>,
    q: Option<&CallbackQuery>,
    text: &str,
    image_id: Option<Uuid>,
    reply_keyboard: InlineKeyboardMarkup,
) -> AppResult<Message> {
    let (chat_id, msg_id) = get_chat_and_msg_id(msg, q).ok_or(AppError::InternalServerError(
        "Failed to get chat id".to_string(),
    ))?;
    let image_bytes = if let Some(image_id) = image_id {
        Some(api_client.get_image_bytes(&image_id).await?)
    } else {
        None
    };
    // Msg id found, try to edit it
    if let Some(msg_id) = msg_id {
        // Telegram does not allow to change type of message
        let is_type_changed = image_bytes.is_some() != has_photo(msg, q);
        if is_type_changed {
            if let Err(e) = bot.delete_message(chat_id, msg_id).await {
                tracing::error!("Failed to delete message: {:?}", e);
            }
            return send_msg(bot, chat_id, text, image_bytes, reply_keyboard).await;
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
    send_msg(bot, chat_id, text, image_bytes, reply_keyboard).await
}

/// Send message with or without photo
async fn send_msg(
    bot: Bot,
    chat_id: ChatId,
    text: &str,
    image_bytes: Option<Bytes>,
    reply_keyboard: InlineKeyboardMarkup,
) -> AppResult<Message> {
    match image_bytes {
        Some(image_bytes) => send_media_msg(bot, chat_id, text, image_bytes, reply_keyboard).await,
        None => send_text_msg(bot, chat_id, text, reply_keyboard).await,
    }
}

/// Check if message has photo
fn has_photo(msg: Option<&Message>, q: Option<&CallbackQuery>) -> bool {
    if let Some(msg) = msg {
        return msg.photo().is_some();
    }
    if let Some(q) = q
        && let Some(MaybeInaccessibleMessage::Regular(msg)) = &q.message
    {
        return msg.photo().is_some();
    }
    false
}

pub fn get_chat_and_msg_id(
    msg: Option<&Message>,
    q: Option<&CallbackQuery>,
) -> Option<(ChatId, Option<MessageId>)> {
    if let Some(msg) = msg {
        return Some((msg.chat.id, Some(msg.id)));
    }
    if let Some(q) = q {
        let chat_id = q.chat_id()?;
        let message_id = match &q.message {
            Some(MaybeInaccessibleMessage::Regular(msg)) => Some(msg.id),
            Some(MaybeInaccessibleMessage::Inaccessible(_)) => None,
            None => None,
        };
        return Some((chat_id, message_id));
    }
    None
}

/// Try edit message with photo. If edit is failed, send new message
async fn edit_media_msg(
    bot: Bot,
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
    send_media_msg(bot, chat_id, text, image_bytes, reply_keyboard).await
}

/// Try edit text message. If edit is failed, send new message
async fn edit_text_msg(
    bot: Bot,
    chat_id: ChatId,
    msg_id: Option<MessageId>,
    text: &str,
    reply_keyboard: InlineKeyboardMarkup,
) -> AppResult<Message> {
    if let Some(msg_id) = msg_id
        && let Ok(msg) = bot
            .edit_message_text(chat_id, msg_id, text)
            .reply_markup(reply_keyboard.clone())
            .send()
            .await
    {
        return Ok(msg);
    }
    send_text_msg(bot, chat_id, text, reply_keyboard).await
}

/// Send message with photo
async fn send_media_msg(
    bot: Bot,
    chat_id: ChatId,
    text: &str,
    image_bytes: Bytes,
    reply_keyboard: InlineKeyboardMarkup,
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
    bot: Bot,
    chat_id: ChatId,
    text: &str,
    reply_keyboard: InlineKeyboardMarkup,
) -> AppResult<Message> {
    Ok(bot
        .send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(reply_keyboard)
        .send()
        .await?)
}
