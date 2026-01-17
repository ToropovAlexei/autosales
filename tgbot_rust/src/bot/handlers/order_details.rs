use std::sync::Arc;

use teloxide::{
    Bot,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        CallbackData, MyDialogue,
        utils::{MessageImage, MsgBy, edit_msg},
    },
    errors::AppResult,
};

pub async fn order_details_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    id: i64,
) -> AppResult<()> {
    let order = api_client.get_order(id).await?;

    let mut msg = format!(
        "📦 <b>Заказ №{}</b>\n\
         💰 Сумма: <b>{} {}</b>\n\
         📅 Дата: {}\n\
         🛍️ <b>Товары:</b>\n",
        order.id,
        order.amount,
        order.currency,
        order.created_at.format("%d.%m.%Y %H:%M"),
    );

    for item in &order.order_items {
        let total_price = item.price_at_purchase * f64::from(item.quantity);
        msg.push_str(&format!(
            "\n• <b>{}</b> × {} — {} {}\n",
            item.name_at_purchase, item.quantity, total_price, order.currency
        ));

        if let Some(ref content) = item.fulfillment_content {
            msg.push_str(&format!("<pre>🔑 Ваш товар:\n{}</pre>\n", content));
        }

        if let Some(ref details) = item.details {
            msg.push_str(&format!(
                "<b>Подробности:</b>\n{}",
                serde_json::to_string_pretty(details).unwrap_or_default()
            ));
        }
    }

    let image = order
        .order_items
        .iter()
        .find_map(|item| item.fulfillment_image_id.map(MessageImage::Uuid));

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        &msg,
        image,
        InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
            "⬅️ Назад",
            CallbackData::ToMyOrders,
        )]]),
    )
    .await?;

    Ok(())
}
