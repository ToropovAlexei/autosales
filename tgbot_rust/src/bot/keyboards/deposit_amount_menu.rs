use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::{BotState, PaymentAction};

pub fn deposit_amount_menu(gateway: &str) -> InlineKeyboardMarkup {
    let amounts = [100, 500, 1000];

    let keyboard: Vec<Vec<InlineKeyboardButton>> = amounts
        .iter()
        .map(|&amount| {
            vec![InlineKeyboardButton::callback(
                format!("{amount} ₽"),
                BotState::Payment {
                    action: PaymentAction::SelectAmount {
                        amount,
                        gateway: gateway.to_string(),
                    },
                },
            )]
        })
        .collect();

    InlineKeyboardMarkup::new(keyboard)
}
