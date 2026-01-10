use std::sync::Arc;

use teloxide::{Bot, types::CallbackQuery};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        BotState, MyDialogue,
        keyboards::deposit_amount_menu::deposit_amount_menu,
        utils::{MsgBy, edit_msg},
    },
    errors::AppResult,
};

pub async fn deposit_amount_handler(
    bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    _bot_state: BotState,
) -> AppResult<()> {
    edit_msg(
        &api_client,
        &bot,
        &MsgBy::CallbackQuery(&q),
        "Выберите сумму для пополнения:",
        None,
        deposit_amount_menu(),
    )
    .await?;

    Ok(())
}
