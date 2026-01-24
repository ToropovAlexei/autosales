use std::sync::Arc;

use crate::bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard;
use crate::bot::utils::{MsgBy, edit_msg};
use crate::bot::{BotState, BotStep};
use crate::{api::backend_api::BackendApi, bot::MyDialogue, errors::AppResult};
use bytes::Bytes;
use teloxide::Bot;
use teloxide::net::Download;
use teloxide::prelude::Requester;
use teloxide::types::Message;
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

    let file_id = if let Some(document) = msg.document() {
        document.file.id.clone()
    } else if let Some(photo) = msg.photo() {
        match photo.last() {
            Some(photo) => photo.file.id.clone(),
            None => {
                edit_msg(
                    &api_client,
                    &dialogue,
                    &bot,
                    &MsgBy::Message(&msg),
                    "Пожалуйста, прикрепите чек в формате JPG, JPEG, PNG или PDF.",
                    None,
                    back_to_main_menu_inline_keyboard(),
                )
                .await?;

                return Ok(());
            }
        }
    } else {
        edit_msg(
            &api_client,
            &dialogue,
            &bot,
            &MsgBy::Message(&msg),
            "Пожалуйста, прикрепите чек в формате JPG, JPEG, PNG или PDF.",
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
        .submit_payment_receipt_file(invoice_id, downloaded_bytes)
        .await?;

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::Message(&msg),
        "Ссылка на чек успешно отправлена! Ожидайте обновления статуса платежа.",
        None,
        back_to_main_menu_inline_keyboard(),
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
