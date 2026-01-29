use crate::bot::utils::{MsgBy, edit_msg};
use crate::{
    api::backend_api::BackendApi, bot::MyDialogue,
    bot::keyboards::back_to_main_menu::back_to_main_menu_inline_keyboard, errors::AppResult,
};
use shared_dtos::user_subscription::UserSubscriptionDetails;
use std::sync::Arc;
use teloxide::{
    dispatching::dialogue::GetChatId,
    prelude::Bot,
    types::CallbackQuery,
    utils::html::{bold, code_block, italic},
};

pub async fn my_subscriptions_handler(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    api_client: Arc<BackendApi>,
) -> AppResult<()> {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => return Ok(()),
    };

    let msg = match api_client.get_user_subscriptions(chat_id.0).await {
        Ok(subscriptions) => {
            if subscriptions.items.is_empty() {
                "У вас пока нет активных подписок.".to_string()
            } else {
                let mut response_text = format!("{}\n\n", bold("🧾 Ваши подписки:"));

                for sub in subscriptions.items {
                    let product_name = sub
                        .product_name
                        .unwrap_or_else(|| format!("Подписка #{}", sub.id));
                    let started = sub.started_at.format("%d.%m.%Y %H:%M").to_string();
                    let expires = sub.expires_at.format("%d.%m.%Y %H:%M").to_string();
                    let next_charge = sub
                        .next_charge_at
                        .map(|v| v.format("%d.%m.%Y %H:%M").to_string());

                    let status = if sub.cancelled_at.is_some() {
                        "🚫 Отменена"
                    } else if sub.expires_at > chrono::Utc::now() {
                        "✅ Активна"
                    } else {
                        "⏳ Истекла"
                    };

                    response_text.push_str(&format!("🔹 {}\n", bold(&product_name)));
                    response_text.push_str(&format!("   {} до {}\n", status, italic(&expires)));
                    response_text.push_str(&format!("   Старт: {}\n", italic(&started)));
                    response_text.push_str(&format!(
                        "   Период: {} дней • Цена: {:.2}\n",
                        sub.period_days, sub.price_at_subscription
                    ));
                    if let Some(next_charge) = next_charge {
                        response_text.push_str(&format!(
                            "   Следующее списание: {}\n",
                            italic(&next_charge)
                        ));
                    }

                    if let Some(details) = sub.details {
                        match details {
                            UserSubscriptionDetails::ContMs {
                                host,
                                port,
                                username,
                                password,
                            } => {
                                response_text.push_str(&format!("   {}\n", bold("🔐 Доступ:")));
                                let address = format!("{}:{}", host, port);
                                let access = format!(
                                    "{}\nlogin: {}\npassword: {}",
                                    address, username, password
                                );
                                response_text.push_str(&format!("{}\n", code_block(&access)));
                            }
                        }
                    }

                    response_text.push('\n');
                }

                response_text
            }
        }
        Err(err) => {
            tracing::error!("Error getting user subscriptions: {err}");
            "Произошла ошибка при получении подписок. Попробуйте позже.".to_string()
        }
    };

    edit_msg(
        &api_client,
        &dialogue,
        &bot,
        &MsgBy::CallbackQuery(&q),
        &msg,
        None,
        back_to_main_menu_inline_keyboard(),
    )
    .await?;

    Ok(())
}
