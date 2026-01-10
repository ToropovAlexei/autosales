use std::sync::Arc;

use teloxide::{Bot, types::CallbackQuery};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        MyDialogue,
        keyboards::payment_gateways_menu::payment_gateways_menu,
        utils::{MsgBy, edit_msg},
    },
    errors::AppResult,
};

pub async fn deposit_gateway_handler(
    bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let payment_gateways = api_client.get_payment_gateways().await?;
    let settings = api_client.get_settings().await?;

    edit_msg(
        &api_client,
        &bot,
        &MsgBy::CallbackQuery(&q),
        "💰 Выберите способ пополнения:",
        None,
        payment_gateways_menu(payment_gateways.items, &settings),
    )
    .await?;

    Ok(())
}
