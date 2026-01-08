use std::sync::Arc;

use teloxide::dispatching::dialogue::GetChatId;
use teloxide::payloads::SendPhotoSetters;
use teloxide::prelude::Requester;
use teloxide::{
    Bot,
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::Request,
    types::{CallbackQuery, InputFile, MaybeInaccessibleMessage, ParseMode},
};

use crate::models::product::Product;
use crate::{
    api::backend_api::BackendApi,
    bot::keyboards::{
        back_to_main_menu::back_to_main_menu_inline_keyboard,
        product_card::product_card_inline_keyboard,
    },
    errors::AppResult,
};

pub async fn product_handler(
    bot: Bot,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    id: i64,
) -> AppResult<()> {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => return Ok(()),
    };
    let message_id = match &q.message {
        Some(MaybeInaccessibleMessage::Regular(msg)) => msg.id,
        Some(MaybeInaccessibleMessage::Inaccessible(_)) => return Ok(()),
        None => return Ok(()),
    };

    let product_result = api_client.get_product(id).await;

    match product_result {
        Ok(product) => {
            display_product(bot, q, api_client, &product).await?;
        }
        Err(err) => {
            tracing::error!("Error getting product: {}", err);
            bot.edit_message_text(
                chat_id,
                message_id,
                "Что-то пошло не так. Попробуйте позже.",
            )
            .reply_markup(back_to_main_menu_inline_keyboard())
            .parse_mode(ParseMode::Html)
            .send()
            .await?;
        }
    };

    Ok(())
}

async fn display_product(
    bot: Bot,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    product: &Product,
) -> AppResult<()> {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => return Ok(()),
    };
    let message_id = match &q.message {
        Some(MaybeInaccessibleMessage::Regular(msg)) => msg.id,
        Some(MaybeInaccessibleMessage::Inaccessible(_)) => return Ok(()),
        None => return Ok(()),
    };

    let caption = format!(
        "<b>{}</b>

<i>Цена:</i> {} ₽",
        product.name, product.price
    );

    let reply_markup = product_card_inline_keyboard(product);

    let image_bytes = if let Some(image_id) = &product.image_id {
        match api_client.get_image_bytes(image_id).await {
            Ok(bytes) => Some(bytes),
            Err(e) => {
                tracing::error!("Failed to get image bytes: {}", e);
                None
            }
        }
    } else {
        None
    };

    bot.delete_message(chat_id, message_id).await.ok();

    if let Some(image_bytes) = image_bytes {
        bot.send_photo(chat_id, InputFile::memory(image_bytes))
            .caption(caption)
            .reply_markup(reply_markup)
            .parse_mode(ParseMode::Html)
            .await?;
    } else {
        bot.send_message(chat_id, caption)
            .reply_markup(reply_markup)
            .parse_mode(ParseMode::Html)
            .await?;
    }

    Ok(())
}
