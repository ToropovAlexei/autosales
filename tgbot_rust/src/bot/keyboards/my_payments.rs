use shared_dtos::invoice::{InvoiceStatus, PaymentInvoiceBotResponse};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::CallbackData;

pub fn my_payments_inline_keyboard(
    active_payments: &Vec<&PaymentInvoiceBotResponse>,
) -> InlineKeyboardMarkup {
    let mut buttons = active_payments
        .iter()
        .filter_map(|payment| match payment.status {
            InvoiceStatus::Pending => Some(vec![InlineKeyboardButton::callback(
                format!("Посмотреть счет #{}", payment.id),
                CallbackData::ToDepositConfirm { id: payment.id },
            )]),
            InvoiceStatus::AwaitingReceipt => Some(vec![InlineKeyboardButton::callback(
                format!("Отправить чек #{}", payment.id),
                CallbackData::ToReceiptRequested { id: payment.id },
            )]),
            _ => None,
        })
        .collect::<Vec<_>>();

    buttons.push(vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        CallbackData::ToMainMenu,
    )]);

    InlineKeyboardMarkup::new(buttons)
}
