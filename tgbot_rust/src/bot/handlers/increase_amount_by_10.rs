use std::sync::Arc;

use teloxide::{
    Bot,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        BotState, CallbackData, MyDialogue,
        utils::{MsgBy, edit_msg},
    },
    errors::AppResult,
};

pub async fn increase_amount_by_10_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg_by: &MsgBy<'_>,
    api_client: Arc<BackendApi>,
    _bot_state: BotState,
    amount: i64,
) -> AppResult<()> {
    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        msg_by,
        "<b>Измените сумму на +10 рублей</b>\n\
На данный момент все реквизиты на вашу сумму заняты, измените сумму на 10 рублей",
        None,
        InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
            format!("Изменить сумму на {} рублей", amount + 10),
            CallbackData::IncreaseAmountBy10,
        )]]),
    )
    .await?;

    Ok(())
}
