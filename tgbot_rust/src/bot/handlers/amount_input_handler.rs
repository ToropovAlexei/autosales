use std::sync::Arc;

use crate::AppState;
use crate::bot::handlers::deposit_confirm::deposit_confirm_handler;
use crate::bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard;
use crate::bot::keyboards::deposit_amount_menu::deposit_amount_menu;
use crate::bot::utils::{MsgBy, edit_msg};
use crate::bot::{BotState, BotStep};
use crate::{api::backend_api::BackendApi, bot::MyDialogue, errors::AppResult};
use teloxide::Bot;
use teloxide::types::Message;

pub async fn amount_input_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    api_client: Arc<BackendApi>,
    bot_state: BotState,
    app_state: AppState,
) -> AppResult<()> {
    let gateway = match bot_state {
        BotState {
            step: BotStep::DepositSelectAmount { gateway },
            ..
        } => gateway,
        _ => return Ok(()),
    };
    let amount = if let Some(amount) = msg.text()
        && let Ok(amount) = amount.parse::<i64>()
        && amount > 0
    {
        amount
    } else {
        edit_msg(
            &api_client,
            &dialogue,
            &bot,
            &MsgBy::Message(&msg),
            "Неверная сумма. Введите число.",
            None,
            deposit_amount_menu(),
        )
        .await?;

        return Ok(());
    };

    let new_state = BotState {
        step: BotStep::DepositConfirm {
            amount,
            gateway,
            invoice: None,
        },
        ..bot_state
    };

    dialogue.update(new_state.clone()).await?;

    let pending_msg = edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::Message(&msg),
        "Подготовка платежа...",
        None,
        back_to_main_menu_inline_keyboard(),
    )
    .await?;
    deposit_confirm_handler(
        bot.clone(),
        &MsgBy::Message(&pending_msg),
        dialogue,
        api_client,
        new_state,
        app_state,
    )
    .await?;

    Ok(())
}
