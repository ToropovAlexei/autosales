use std::collections::HashMap;

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::callback_data::{CallbackData, PaymentAction};

pub fn payment_gateways_menu(
    gateways: Vec<String>,
    public_settings: HashMap<String, String>,
    instructions_url: Option<&str>,
) -> InlineKeyboardMarkup {
    let mut buttons = Vec::new();
    if let Ok(url) = reqwest::Url::parse(instructions_url.unwrap_or_default()) {
        buttons.push([InlineKeyboardButton::url("ℹ️ Как пополнить баланс?", url)]);
    }

    let mut gateways_with_bonuses: Vec<(String, String, f64)> = Vec::new();

    for gateway in gateways {
        let gateway_clone = gateway.clone();
        let bonus_value = public_settings
            .get(&format!("GATEWAY_BONUS_{}", gateway))
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let display_name = public_settings
            .get(&format!("GATEWAY_DISPLAY_NAME_{}", gateway))
            .unwrap_or(&gateway_clone);
        gateways_with_bonuses.push((gateway, display_name.to_string(), bonus_value));
    }

    gateways_with_bonuses.sort_by(|a, b| {
        b.2.partial_cmp(&a.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.cmp(&b.1))
    });

    for (index, (gateway, display_name, bonus_value)) in gateways_with_bonuses.iter().enumerate() {
        let mut display_name = if bonus_value > &0.0 {
            format!("{} (+{}% бонус)", display_name, bonus_value)
        } else {
            display_name.to_string()
        };
        if index == 0 && bonus_value > &0.0 {
            display_name = format!("🔥🔥 {} 🔥🔥", display_name);
        }
        buttons.push([InlineKeyboardButton::callback(
            display_name,
            CallbackData::Payment {
                action: PaymentAction::SelectGateway {
                    gateway: gateway.to_string(),
                },
            },
        )]);
    }

    buttons.push([InlineKeyboardButton::callback(
        "⬅️ Назад",
        CallbackData::MainMenu,
    )]);
    return InlineKeyboardMarkup::new(buttons);
}
