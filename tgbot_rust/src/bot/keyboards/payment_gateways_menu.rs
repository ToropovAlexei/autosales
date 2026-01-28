use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::{
    bot::CallbackData,
    models::{
        payment::{PaymentGateway, PaymentSystem},
        settings::Settings,
    },
};

pub fn payment_gateways_menu(
    gateways: Vec<PaymentGateway>,
    public_settings: &Settings,
) -> InlineKeyboardMarkup {
    let mut buttons = Vec::new();
    // TODO Instructions must be in settings
    // TODO Temporary disabled
    // if let Ok(url) = reqwest::Url::parse("https://telegra.ph/your-payment-instructions-here") {
    //     buttons.push([InlineKeyboardButton::url("ℹ️ Как пополнить баланс?", url)]);
    // }

    let mut gateways_with_bonuses: Vec<(PaymentSystem, String, f64)> = Vec::new();

    for gateway in gateways {
        let bonus_value = match gateway.name {
            PaymentSystem::Mock => public_settings.pricing_gateway_bonus_mock_provider,
            PaymentSystem::PlatformCard => public_settings.pricing_gateway_bonus_platform_card,
            PaymentSystem::PlatformSBP => public_settings.pricing_gateway_bonus_platform_sbp,
        };
        gateways_with_bonuses.push((gateway.name, gateway.display_name, bonus_value));
    }

    gateways_with_bonuses.sort_by(|a, b| {
        b.2.partial_cmp(&a.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.cmp(&b.1))
    });

    for (index, (gateway, display_name, bonus_value)) in gateways_with_bonuses.iter().enumerate() {
        let mut display_name = if bonus_value > &0.0 {
            format!("{} (+{}% скидка)", display_name, bonus_value)
        } else {
            display_name.to_string()
        };
        if index == 0 && bonus_value > &0.0 {
            display_name = format!("🔥🔥 {} 🔥🔥", display_name);
        }
        buttons.push([InlineKeyboardButton::callback(
            display_name,
            CallbackData::SelectGateway {
                gateway: gateway.clone(),
            },
        )]);
    }

    buttons.push([InlineKeyboardButton::callback(
        "⬅️ Назад",
        CallbackData::ToMainMenu,
    )]);
    InlineKeyboardMarkup::new(buttons)
}
