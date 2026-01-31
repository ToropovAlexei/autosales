use std::{collections::HashMap, sync::Arc};

use crate::{
    api::backend_api::BackendApi,
    bot::{
        MyDialogue,
        keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard,
        utils::{MsgBy, edit_msg},
    },
    errors::AppResult,
};
use shared_dtos::analytics::BotAnalyticsBotResponse;
use teloxide::{dispatching::dialogue::GetChatId, prelude::*};

pub async fn bot_stats_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let chat_id = match q.chat_id() {
        Some(id) => id,
        None => return Ok(()),
    };
    let telegram_id = chat_id.0;

    let stats = api_client.get_referral_stats(telegram_id).await?.items;
    let bots = api_client.get_customer_bots(telegram_id).await?.items;

    if stats.is_empty() {
        bot.answer_callback_query(q.id)
            .text("У вас пока нет статистики.")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    let bots_map: HashMap<i64, _> = bots.into_iter().map(|b| (b.id, b)).collect();

    let mut stats_message = String::from("📊 Статистика по вашим ботам:\n\n");
    let mut total_earnings_by_bots: f64 = 0.0;
    let mut total_purchases_by_bots: i64 = 0;

    for BotAnalyticsBotResponse {
        bot_id,
        purchase_count,
        total_earnings,
    } in stats
    {
        if let Some(bot_info) = bots_map.get(&bot_id) {
            let bot_username = if !bot_info.username.is_empty() {
                bot_info.username.clone()
            } else {
                bot_info
                    .token
                    .split(':')
                    .next()
                    .unwrap_or("unknown")
                    .to_string()
            };

            stats_message.push_str(&format!("🤖 @{}:\n", bot_username));
            stats_message.push_str(&format!(
                "    - 💰 Заработано: {:.2} руб.\n",
                total_earnings
            ));
            stats_message.push_str(&format!("    - 🛒 Продаж: {}\n\n", purchase_count));

            total_earnings_by_bots += total_earnings;
            total_purchases_by_bots += purchase_count;
        }
    }

    stats_message.push_str("🏆 Итого:\n");
    stats_message.push_str(&format!(
        "    - 💰 Заработано: {:.2} руб.\n",
        total_earnings_by_bots
    ));
    stats_message.push_str(&format!("    - 🛒 Продаж: {}", total_purchases_by_bots));

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        &stats_message,
        None,
        back_to_main_menu_inline_keyboard(),
    )
    .await?;

    Ok(())
}
