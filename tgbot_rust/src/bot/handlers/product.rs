use std::sync::Arc;

use shared_dtos::product::ProductBotResponse;
use teloxide::{Bot, types::CallbackQuery};

use crate::bot::MyDialogue;
use crate::bot::utils::{MessageImage, MsgBy, edit_msg};
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
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    id: i64,
) -> AppResult<()> {
    let product_result = api_client.get_product(id).await;

    match product_result {
        Ok(product) => {
            display_product(bot, dialogue, q, api_client, &product).await?;
        }
        Err(err) => {
            tracing::error!("Error getting product: {}", err);
            edit_msg(
                &api_client,
                &dialogue,
                &bot,
                &MsgBy::CallbackQuery(&q),
                "Что-то пошло не так. Попробуйте позже.",
                None,
                back_to_main_menu_inline_keyboard(),
            )
            .await?;
        }
    };

    Ok(())
}

async fn display_product(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    product: &ProductBotResponse,
) -> AppResult<()> {
    let caption = format!(
        "<b>{}</b>

<i>Цена:</i> {} ₽",
        product.name, product.price
    );

    let reply_markup = product_card_inline_keyboard(product);

    let image_bytes = if let Some(image_id) = &product.image_id {
        match api_client.get_image_bytes(image_id).await {
            Ok(bytes) => Some(MessageImage::Bytes(bytes)),
            Err(e) => {
                tracing::error!("Failed to get image bytes: {}", e);
                None
            }
        }
    } else {
        None
    };

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        &caption,
        image_bytes,
        reply_markup,
    )
    .await?;

    Ok(())
}
