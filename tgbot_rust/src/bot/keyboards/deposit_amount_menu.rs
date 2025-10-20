use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::callback_data::{CallbackData, PaymentAction};

pub fn deposit_amount_menu(gateway: &str) -> InlineKeyboardMarkup {
    let amounts = [100, 500, 1000];

    let keyboard: Vec<Vec<InlineKeyboardButton>> = amounts
        .iter()
        .map(|&amount| {
            vec![InlineKeyboardButton::callback(
                format!("{amount} ₽"),
                CallbackData::Payment {
                    action: PaymentAction::SelectAmount,
                    gateway: gateway.to_string(),
                    amount,
                },
            )]
        })
        .collect();

    InlineKeyboardMarkup::new(keyboard)
}
