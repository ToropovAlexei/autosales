use std::sync::Arc;

use crate::api::backend_api::BackendApi;
use crate::bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard;
use crate::bot::utils::{MsgBy, edit_msg};
use crate::bot::{BotState, BotStep, CallbackData, InvoiceData, MyDialogue};
use crate::errors::AppResult;
use shared_dtos::invoice::PaymentDetails;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use url::Url;

pub async fn deposit_confirm_handler(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    api_client: Arc<BackendApi>,
    bot_state: BotState,
) -> AppResult<()> {
    let (gateway, amount, invoice_data) = match bot_state.step {
        BotStep::DepositConfirm {
            gateway,
            amount,
            invoice,
        } => (gateway, amount, invoice),
        _ => {
            tracing::error!("Expected DepositConfirm bot state");
            return Ok(());
        }
    };

    let invoice_data = match invoice_data {
        Some(data) => data,
        None => {
            let telegram_id = dialogue.chat_id().0;
            let response = match api_client
                .create_deposit_invoice(&gateway, amount as f64, telegram_id)
                .await
            {
                Ok(res) => res,
                Err(err) => {
                    tracing::error!("Error creating invoice: {err}");
                    edit_msg(
                        &api_client,
                        &dialogue,
                        &bot,
                        &MsgBy::CallbackQuery(&q),
                        "Что-то пошло не так. Попробуйте ещё раз.",
                        None,
                        back_to_main_menu_inline_keyboard(),
                    )
                    .await?;

                    return Ok(());
                }
            };
            InvoiceData {
                id: response.id,
                details: response.payment_details,
            }
        }
    };

    dialogue
        .update(BotState {
            step: BotStep::DepositConfirm {
                gateway,
                amount,
                invoice: Some(invoice_data.clone()),
            },
            ..bot_state
        })
        .await?;

    let text = match &invoice_data.details {
        None => "Не удалось получить реквизиты для оплаты. Попробуйте другой способ.".to_string(),
        Some(details) => match details {
            PaymentDetails::Mock { .. } => format!(
                "✅ Ваш счет на {} ₽ создан.\n\nНажмите на кнопку ниже, чтобы перейти к оплате.",
                amount
            ),
            PaymentDetails::PlatformCard {
                bank_name,
                account_name,
                card_number,
                amount,
            } => format!(
                "Реквизиты для оплаты:\n\n\
                 <b>Банк:</b> {}\n\
                 <b>Номер карты:</b> {}\n\
                 <b>Получатель:</b> {}\n\
                 <b>Сумма:</b> {} ₽\n\n\
                 На оплату дается 30 минут!\n\
                 В случае, если вы не оплатите в течении 30 минут, платеж не будет зачислен!\n\
                 После оплаты ОБЯЗАТЕЛЬНО НАЖМИТЕ \"Оплатил\"",
                bank_name, card_number, account_name, amount
            ),
            PaymentDetails::PlatformSBP {
                bank_name,
                account_name,
                sbp_number,
                amount,
            } => format!(
                "Реквизиты для оплаты:\n\n\
                 <b>Банк:</b> {}\n\
                 <b>Номер СБП:</b> {}\n\
                 <b>Получатель:</b> {}\n\
                 <b>Сумма:</b> {} ₽\n\n\
                 На оплату дается 30 минут!\n\
                 В случае, если вы не оплатите в течении 30 минут, платеж не будет зачислен!\n\
                 После оплаты ОБЯЗАТЕЛЬНО НАЖМИТЕ \"Оплатил\"",
                bank_name, sbp_number, account_name, amount
            ),
        },
    };

    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    if let Some(details) = &invoice_data.details {
        match details {
            PaymentDetails::Mock { pay_url } => {
                if let Ok(pay_url) = Url::parse(pay_url.as_str()) {
                    keyboard.push(vec![InlineKeyboardButton::url("Оплатить", pay_url)]);
                }
            }
            PaymentDetails::PlatformCard { .. } | PaymentDetails::PlatformSBP { .. } => {
                keyboard.push(vec![InlineKeyboardButton::callback(
                    "Оплатил",
                    CallbackData::ConfirmPayment {
                        id: invoice_data.id,
                    },
                )]);
            }
        }
    }

    keyboard.push(vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        CallbackData::ToMainMenu,
    )]);

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        &text,
        None,
        InlineKeyboardMarkup::new(keyboard),
    )
    .await?;

    Ok(())
}
