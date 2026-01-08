use std::sync::Arc;

use teloxide::{
    Bot,
    payloads::{EditMessageMediaSetters, EditMessageTextSetters, SendMessageSetters},
    prelude::Request,
    types::{
        CallbackQuery, InputFile, InputMedia, InputMediaPhoto, MaybeInaccessibleMessage, ParseMode,
    },
};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        MyDialogue,
        keyboards::{
            back_to_main_menu::back_to_main_menu_inline_keyboard,
            catalog_menu::catalog_menu_inline_keyboard,
        },
    },
    errors::AppResult,
    models::category::Category,
};
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::payloads::SendPhotoSetters;
use teloxide::prelude::Requester;

pub async fn catalog_handler(
    bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    category_id: Option<i64>,
) -> AppResult<()> {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => {
            tracing::error!("No chat id found");
            return Ok(());
        }
    };
    let (message_id, has_photo) = match &q.message {
        Some(MaybeInaccessibleMessage::Regular(msg)) => (msg.id, msg.photo().is_some()),
        Some(MaybeInaccessibleMessage::Inaccessible(_)) => {
            tracing::error!("Inaccessible message found");
            return Ok(());
        }
        None => {
            tracing::error!("No message found");
            return Ok(());
        }
    };

    let categories = match api_client.get_categories().await {
        Ok(categories) => categories,
        Err(err) => {
            tracing::error!("Error getting categories: {}", err);
            bot.edit_message_text(
                chat_id,
                message_id,
                "Что-то пошло не так. Попробуйте позже.",
            )
            .reply_markup(back_to_main_menu_inline_keyboard())
            .parse_mode(ParseMode::Html)
            .send()
            .await?;
            return Ok(());
        }
    };

    let category = match category_id {
        Some(category_id) => categories.items.iter().find(|c| c.id == category_id),
        None => None,
    };

    let categories_to_show = categories
        .items
        .iter()
        .filter(|c| match category {
            Some(category) => c.parent_id == Some(category.id),
            None => c.parent_id.is_none(),
        })
        .cloned()
        .collect::<Vec<Category>>();

    let image_bytes = match category {
        Some(category) => match &category.image_id {
            Some(image_id) => match api_client.get_image_bytes(image_id).await {
                Ok(image_bytes) => Some(image_bytes),
                Err(err) => {
                    tracing::error!("Error getting image: {}", err);
                    None
                }
            },
            None => None,
        },
        None => None,
    };

    let products = match category_id {
        None => vec![],
        Some(category_id) => match api_client.get_products(category_id).await {
            Ok(products) => products.items,
            Err(err) => {
                tracing::error!("Error getting products: {}", err);
                bot.edit_message_text(
                    chat_id,
                    message_id,
                    "Что-то пошло не так. Попробуйте позже.",
                )
                .reply_markup(back_to_main_menu_inline_keyboard())
                .parse_mode(ParseMode::Html)
                .send()
                .await?;
                return Ok(());
            }
        },
    };

    let caption = "🛍️ Выберите товар или категорию:";
    let reply_markup = catalog_menu_inline_keyboard(
        categories_to_show.as_slice(),
        &products,
        category_id,
        category.and_then(|c| c.parent_id),
    );

    if let Some(image_bytes) = image_bytes {
        if has_photo {
            let input_media = InputMediaPhoto::new(InputFile::memory(image_bytes))
                .caption(caption)
                .parse_mode(ParseMode::Html);
            bot.edit_message_media(chat_id, message_id, InputMedia::Photo(input_media))
                .reply_markup(reply_markup)
                .await?;
        } else {
            bot.send_photo(chat_id, InputFile::memory(image_bytes))
                .caption(caption)
                .reply_markup(reply_markup)
                .await?;
            bot.delete_message(chat_id, message_id).await?;
        }
    } else if !has_photo {
        bot.edit_message_text(chat_id, message_id, caption)
            .reply_markup(reply_markup)
            .parse_mode(ParseMode::Html)
            .await?;
    } else {
        bot.send_message(chat_id, caption)
            .reply_markup(reply_markup)
            .parse_mode(ParseMode::Html)
            .await?;
        bot.delete_message(chat_id, message_id).await?;
    }
    Ok(())
}
