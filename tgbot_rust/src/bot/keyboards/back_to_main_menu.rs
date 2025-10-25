use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::CallbackData;

pub fn back_to_main_menu_inline_keyboard() -> InlineKeyboardMarkup {
    let buttons = vec![[InlineKeyboardButton::callback(
        "⬅️ Главное меню",
        CallbackData::ToMainMenu,
    )]];
    InlineKeyboardMarkup::new(buttons)
}
