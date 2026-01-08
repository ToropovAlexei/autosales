use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::CallbackData;

pub fn main_menu_inline_keyboard(show_referral_program: bool) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![
        vec![InlineKeyboardButton::callback(
            "🛍️ Каталог",
            CallbackData::ToCategory { category_id: None },
        )],
        vec![InlineKeyboardButton::callback(
            "💳 Баланс",
            CallbackData::ToBalance,
        )],
        vec![InlineKeyboardButton::callback(
            "🧾 Мои заказы",
            CallbackData::ToMyOrders,
        )],
        vec![InlineKeyboardButton::callback(
            "🧾 Мои подписки",
            CallbackData::ToMySubscriptions,
        )],
        vec![InlineKeyboardButton::callback(
            "💰 Пополнить баланс",
            CallbackData::ToDepositSelectGateway,
        )],
    ];

    if show_referral_program {
        keyboard.push(vec![InlineKeyboardButton::callback(
            "🤝 Реферальный магазин",
            CallbackData::ToReferralProgram,
        )]);
    }

    keyboard.push(vec![InlineKeyboardButton::callback(
        "💬 Поддержка",
        CallbackData::ToSupport,
    )]);

    InlineKeyboardMarkup::new(keyboard)
}
