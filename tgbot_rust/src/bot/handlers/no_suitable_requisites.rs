use std::sync::Arc;

use teloxide::{
    Bot,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        CallbackData, MyDialogue,
        utils::{MsgBy, edit_msg},
    },
    errors::AppResult,
};

pub async fn no_suitable_requisites_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg_by: &MsgBy<'_>,
    api_client: Arc<BackendApi>,
    instructions_url: &str,
) -> AppResult<()> {
    // TODO not mock provider. but crypto
    let percent = api_client
        .get_settings()
        .await?
        .pricing_gateway_bonus_mock_provider;
    let mut buttons = Vec::new();
    if let Ok(url) = reqwest::Url::parse(instructions_url) {
        buttons.push([InlineKeyboardButton::url("ℹ️ Как пополнить баланс?", url)]);
    }
    buttons.push([InlineKeyboardButton::callback(
        "⬅️ Главное меню",
        CallbackData::ToMainMenu,
    )]);
    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        msg_by,
        &format!(
            "На данный момент все реквизиты заняты\n\
        Но вы можете пополнить баланс в криптовалюте  USDT trc20\n\
        Так вы получите +{percent}% к балансу!"
        ),
        None,
        InlineKeyboardMarkup::new(buttons),
    )
    .await?;

    Ok(())
}
