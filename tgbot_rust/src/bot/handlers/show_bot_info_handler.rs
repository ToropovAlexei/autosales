use std::sync::Arc;

use crate::{api::backend_api::BackendApi, bot::MyDialogue, errors::AppResult};
use teloxide::prelude::*;

pub async fn show_bot_info_handler(
    bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
    id: i64,
) -> AppResult<()> {
    let bot_info = api_client.get_bot(id).await?;

    let primary_status = if bot_info.is_primary {
        "Основной"
    } else {
        "Резервный"
    };
    let active_status = if bot_info.is_active {
        "Активен"
    } else {
        "Неактивен"
    };

    let text = format!(
        "Бот @{}\nСтатус: {}, {}\nПроцент: {}%",
        bot_info.username, active_status, primary_status, bot_info.referral_percentage
    );

    bot.answer_callback_query(q.id)
        .text(text)
        .show_alert(true)
        .await?;

    Ok(())
}
