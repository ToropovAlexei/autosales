use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::{bot::CallbackData, models::Product};

pub fn product_card_inline_keyboard(product: &Product) -> InlineKeyboardMarkup {
    let buy_button =
        InlineKeyboardButton::callback("✅ Купить", CallbackData::Buy { id: product.id });

    let back_button = InlineKeyboardButton::callback(
        "⬅️ Назад к товарам",
        CallbackData::ToCategory {
            category_id: product.category_id,
        },
    );

    InlineKeyboardMarkup::new(vec![vec![buy_button], vec![back_button]])
}
