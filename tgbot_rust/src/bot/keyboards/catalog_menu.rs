use shared_dtos::{category::CategoryBotResponse, product::ProductBotResponse};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::CallbackData;

pub fn catalog_menu_inline_keyboard(
    categories: &[CategoryBotResponse],
    products: &[ProductBotResponse],
    category_id: Option<i64>,
    parent_category_id: Option<i64>,
) -> InlineKeyboardMarkup {
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = categories
        .iter()
        .map(|category| {
            vec![InlineKeyboardButton::callback(
                category.name.clone(),
                CallbackData::ToCategory {
                    category_id: Some(category.id),
                },
            )]
        })
        .collect();

    products.iter().for_each(|product| {
        buttons.push(vec![InlineKeyboardButton::callback(
            format!("🔹 {} - {} ₽", product.name, product.price.ceil()),
            CallbackData::ToProduct { id: product.id },
        )])
    });

    let callback_data = match category_id {
        None => CallbackData::ToMainMenu,
        Some(_) => CallbackData::ToCategory {
            category_id: parent_category_id,
        },
    };

    buttons.push(vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        callback_data,
    )]);

    InlineKeyboardMarkup::new(buttons)
}
