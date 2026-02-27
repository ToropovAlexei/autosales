use std::sync::Arc;

use crate::AppState;
use crate::api::api_errors::ApiClientError;
use crate::api::backend_api::BackendApi;
use crate::bot::handlers::increase_amount_by_10::increase_amount_by_10_handler;
use crate::bot::handlers::no_suitable_requisites::no_suitable_requisites_handler;
use crate::bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard;
use crate::bot::utils::{MsgBy, build_invoice_payment_text, edit_msg};
use crate::bot::{BotState, BotStep, CallbackData, InvoiceData, MyDialogue};
use crate::errors::AppResult;
use shared_dtos::invoice::PaymentDetails;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use url::Url;

pub async fn deposit_confirm_handler(
    bot: Bot,
    msg_by: &MsgBy<'_>,
    dialogue: MyDialogue,
    api_client: Arc<BackendApi>,
    bot_state: BotState,
    app_state: AppState,
) -> AppResult<()> {
    let (gateway, amount, invoice_data) = match bot_state.clone().step {
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
                    if let ApiClientError::Unsuccessful(err) = err {
                        if err.contains("Increase amount by 10") {
                            increase_amount_by_10_handler(
                                bot, dialogue, msg_by, api_client, bot_state, amount,
                            )
                            .await?;
                            return Ok(());
                        }
                        if err.contains("No suitable requisites") {
                            no_suitable_requisites_handler(
                                bot,
                                dialogue,
                                msg_by,
                                api_client,
                                &app_state.config.payment_instructions_url,
                            )
                            .await?;
                            return Ok(());
                        }
                    }
                    edit_msg(
                        &api_client,
                        &dialogue,
                        &bot,
                        msg_by,
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
                gateway_invoice_id: response.gateway_invoice_id,
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

    let text = build_invoice_payment_text(&invoice_data, amount, None);

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
                keyboard.push(vec![InlineKeyboardButton::callback(
                    "Отменить платеж",
                    CallbackData::CancelPayment {
                        id: invoice_data.id,
                    },
                )]);
                keyboard.push(vec![InlineKeyboardButton::callback(
                    "Связаться с поддержкой",
                    CallbackData::ToSupport,
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
        msg_by,
        &text,
        None,
        InlineKeyboardMarkup::new(keyboard),
    )
    .await?;

    Ok(())
}
