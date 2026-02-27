use std::sync::Arc;

use crate::bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard;
use crate::bot::utils::{MsgBy, build_receipt_upload_instruction_text, edit_msg};
use crate::bot::{BotState, BotStep, CallbackData};
use crate::{api::backend_api::BackendApi, bot::MyDialogue, errors::AppResult};
use bytes::Bytes;
use mime;
use teloxide::Bot;
use teloxide::net::Download;
use teloxide::prelude::Requester;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};
use tokio_stream::StreamExt;

pub async fn receipt_submitted_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    api_client: Arc<BackendApi>,
    bot_state: BotState,
) -> AppResult<()> {
    let invoice_id = match bot_state.step {
        BotStep::ReceiptRequested { invoice_id } => invoice_id,
        _ => return Ok(()),
    };

    let (file_id, file_name, content_type) = if let Some(document) = msg.document()
        && let Some(mime_type) = &document.mime_type
        && matches!(
            (mime_type.type_(), mime_type.subtype()),
            (mime::APPLICATION, mime::PDF)
        ) {
        (
            document.file.id.clone(),
            document.file_name.clone(),
            document.mime_type.clone().map(|e| e.to_string()),
        )
    } else {
        let text = build_receipt_upload_instruction_text(None, true);
        edit_msg(
            &api_client,
            &dialogue,
            &bot,
            &MsgBy::Message(&msg),
            &text,
            None,
            back_to_main_menu_inline_keyboard(),
        )
        .await?;

        return Ok(());
    };

    let tg_file = bot.get_file(file_id).await?;
    let mut downloaded_bytes = Bytes::new();
    let mut stream = bot.download_file_stream(&tg_file.path);

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(err) => {
                tracing::error!("Failed to download file chunk: {err}");
                edit_msg(
                    &api_client,
                    &dialogue,
                    &bot,
                    &MsgBy::Message(&msg),
                    "При получении чека произошла ошибка. Пожалуйста, попробуйте ещё раз.",
                    None,
                    back_to_main_menu_inline_keyboard(),
                )
                .await?;
                return Ok(());
            }
        };
        downloaded_bytes = Bytes::from([downloaded_bytes.as_ref(), chunk.as_ref()].concat());
    }

    api_client
        .submit_payment_receipt_file(
            invoice_id,
            downloaded_bytes,
            file_name,
            content_type.as_deref(),
        )
        .await?;

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::Message(&msg),
        "Чек успешно отправлен.\n\
         Ожидание решения по платежу.\n\
         Максимальное время ожидания 30 минут.",
        None,
        InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback(
                "Главное меню",
                CallbackData::ToMainMenu,
            )],
            vec![InlineKeyboardButton::callback(
                "Связаться с оператором",
                CallbackData::ToSupport,
            )],
        ]),
    )
    .await?;

    dialogue
        .update(BotState {
            step: BotStep::ReceiptSubmitted { invoice_id },
            ..bot_state
        })
        .await?;

    Ok(())
}
