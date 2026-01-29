use shared_dtos::order::EnrichedOrderBotResponse;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::CallbackData;

pub fn my_orders_inline_keyboard(orders: &[EnrichedOrderBotResponse]) -> InlineKeyboardMarkup {
    let mut buttons = orders
        .iter()
        .map(|order| {
            let created_formatted = order.created_at.format("%d.%m.%Y %H:%M").to_string();
            let product_name = order
                .order_items
                // TODO First only?
                .first()
                .map(|o| o.name_at_purchase.clone())
                .unwrap_or("Неизвестный товар".to_string());
            let label = format!("{created_formatted} - {product_name}");
            vec![InlineKeyboardButton::callback(
                label,
                CallbackData::ToOrderDetails { id: order.id },
            )]
        })
        .collect::<Vec<_>>();

    buttons.push(vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        CallbackData::ToMainMenu,
    )]);

    InlineKeyboardMarkup::new(buttons)
}
