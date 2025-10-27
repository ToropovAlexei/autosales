use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::{
    bot::{CallbackData, ProductVariant},
    models::{Category, Product},
};

pub fn catalog_menu_inline_keyboard(
    categories: &[Category],
    products: &[Product],
    category_id: i64,
    parent_category_id: i64,
) -> InlineKeyboardMarkup {
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = categories
        .iter()
        .map(|category| {
            vec![InlineKeyboardButton::callback(
                category.name.clone(),
                CallbackData::ToCategory {
                    category_id: category.id,
                },
            )]
        })
        .collect();

    products.iter().for_each(|product| {
        buttons.push(vec![InlineKeyboardButton::callback(
            format!("🔹 {} - {} ₽", product.name, product.price),
            match product.external_id.clone() {
                Some(external_id) => CallbackData::ToProduct {
                    product: ProductVariant::ExternalProduct {
                        id: external_id,
                        provider: product.provider.clone().unwrap_or_default(),
                    },
                },
                None => CallbackData::ToProduct {
                    product: ProductVariant::Product { id: product.id },
                },
            },
        )])
    });

    let callback_data = match category_id {
        0 => CallbackData::ToMainMenu,
        _ => CallbackData::ToCategory {
            category_id: parent_category_id,
        },
    };

    buttons.push(vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        callback_data,
    )]);

    InlineKeyboardMarkup::new(buttons)
}
