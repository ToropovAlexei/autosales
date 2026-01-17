use std::sync::Arc;

use teloxide::{Bot, types::CallbackQuery};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        MyDialogue,
        keyboards::catalog_menu::catalog_menu_inline_keyboard,
        utils::{MessageImage, MsgBy, edit_msg},
    },
    errors::AppResult,
    models::category::Category,
};

pub async fn catalog_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    category_id: Option<i64>,
) -> AppResult<()> {
    let categories = api_client.get_categories().await?;

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
                Ok(image_bytes) => Some(MessageImage::Bytes(image_bytes)),
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
        Some(category_id) => api_client.get_products(category_id).await?.items,
    };

    let caption = "🛍️ Выберите товар или категорию:";
    let reply_markup = catalog_menu_inline_keyboard(
        categories_to_show.as_slice(),
        &products,
        category_id,
        category.and_then(|c| c.parent_id),
    );

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        caption,
        image_bytes,
        reply_markup,
    )
    .await?;

    Ok(())
}
