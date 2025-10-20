use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn back_to_main_menu_inline_keyboard() -> InlineKeyboardMarkup {
    let buttons = vec![[InlineKeyboardButton::callback(
        "⬅️ Главное меню",
        "main_menu",
    )]];
    InlineKeyboardMarkup::new(buttons)
}
