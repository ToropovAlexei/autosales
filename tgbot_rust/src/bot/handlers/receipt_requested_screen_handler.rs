use std::sync::Arc;

use shared_dtos::invoice::InvoiceStatus;
use teloxide::{
    prelude::Bot,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::{
    AppState,
    api::backend_api::BackendApi,
    bot::{
        BotState, BotStep, InvoiceData, MyDialogue,
        handlers::deposit_confirm::deposit_confirm_handler,
        utils::{MsgBy, build_receipt_upload_instruction_text, edit_msg, support_operator_buttons},
    },
    errors::AppResult,
};

pub async fn receipt_requested_screen_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    bot_state: BotState,
    app_state: AppState,
    invoice_id: i64,
) -> AppResult<()> {
    let invoice = api_client.get_invoice(invoice_id).await?;
    if invoice.status != InvoiceStatus::AwaitingReceipt {
        let new_state = BotState {
            step: BotStep::DepositConfirm {
                gateway: invoice.gateway,
                amount: invoice.amount as i64,
                invoice: Some(InvoiceData {
                    id: invoice_id,
                    details: invoice.payment_details,
                    gateway_invoice_id: invoice.gateway_invoice_id,
                }),
            },
            ..bot_state
        };
        dialogue.update(new_state.clone()).await?;
        deposit_confirm_handler(
            bot,
            &MsgBy::CallbackQuery(&q),
            dialogue,
            api_client,
            new_state,
            app_state,
        )
        .await?;
        return Ok(());
    }

    dialogue
        .update(BotState {
            step: BotStep::ReceiptRequested { invoice_id },
            ..bot_state
        })
        .await?;

    let support_operators = api_client
        .get_settings()
        .await?
        .bot_payment_system_support_operators;
    let support_operator_rows = support_operator_buttons(&support_operators);

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        &build_receipt_upload_instruction_text(None, false),
        None,
        InlineKeyboardMarkup::new(
            [vec![InlineKeyboardButton::callback(
                "⬅️ Главное меню",
                crate::bot::CallbackData::ToMainMenu,
            )]]
            .into_iter()
            .chain(support_operator_rows.into_iter())
            .collect::<Vec<_>>(),
        ),
    )
    .await?;

    Ok(())
}
