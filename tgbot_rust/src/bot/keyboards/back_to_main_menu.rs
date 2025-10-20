use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::callback_data::CallbackData;

pub fn back_to_main_menu_inline_keyboard() -> InlineKeyboardMarkup {
    let buttons = vec![[InlineKeyboardButton::callback(
        "⬅️ Главное меню",
        CallbackData::MainMenu,
    )]];
    InlineKeyboardMarkup::new(buttons)
}
