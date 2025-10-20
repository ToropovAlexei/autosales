use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::callback_data::{CallbackData, CategoryAction};

pub fn main_menu_inline_keyboard(show_referral_program: bool) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![
        vec![InlineKeyboardButton::callback(
            "🛍️ Каталог",
            CallbackData::Category {
                action: CategoryAction::View,
                category_id: 0,
            },
        )],
        vec![InlineKeyboardButton::callback(
            "💳 Баланс",
            CallbackData::Balance,
        )],
        vec![InlineKeyboardButton::callback(
            "🧾 Мои заказы",
            CallbackData::MyOrders,
        )],
        vec![InlineKeyboardButton::callback(
            "🧾 Мои подписки",
            CallbackData::MySubscriptions,
        )],
        vec![InlineKeyboardButton::callback(
            "💰 Пополнить баланс",
            CallbackData::Deposit,
        )],
    ];

    if show_referral_program {
        keyboard.push(vec![InlineKeyboardButton::callback(
            "🤝 Реферальный магазин",
            CallbackData::ReferralProgram,
        )]);
    }

    keyboard.push(vec![InlineKeyboardButton::callback(
        "💬 Поддержка",
        CallbackData::Support,
    )]);

    InlineKeyboardMarkup::new(keyboard)
}
