use std::sync::Arc;

use teloxide::{
    Bot,
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::Request,
    types::{CallbackQuery, InputFile, MaybeInaccessibleMessage, ParseMode},
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
    models::Category,
};
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::payloads::SendPhotoSetters;
use teloxide::prelude::Requester;

pub async fn catalog_handler(
    bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    _username: String,
    api_client: Arc<BackendApi>,
    category_id: i64,
) -> AppResult<()> {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => {
            tracing::error!("No chat id found");
            return Ok(());
        }
    };
    let message_id = match &q.message {
        Some(MaybeInaccessibleMessage::Regular(msg)) => msg.id,
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

    let category = find_category_by_id(&categories, category_id);

    let categories_to_show: &[Category] = match category_id {
        0 => &categories,
        _ => category
            .and_then(|c| c.sub_categories.as_deref())
            .unwrap_or(&[]),
    };

    let image_bytes = match category {
        Some(category) => match &category.image_id {
            Some(image_id) => match api_client.get_image_bytes(&image_id).await {
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
        0 => vec![],
        _ => match api_client.get_products(category_id).await {
            Ok(products) => products,
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
        categories_to_show,
        &products,
        category_id,
        category.and_then(|x| x.parent_id).unwrap_or_default(),
    );
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

fn find_category_by_id(categories: &[Category], category_id: i64) -> Option<&Category> {
    categories.iter().find_map(|category| {
        if category.id == category_id {
            return Some(category);
        }
        if let Some(sub) = &category.sub_categories {
            if let Some(found) = find_category_by_id(sub, category_id) {
                return Some(found);
            }
        }
        None
    })
}
