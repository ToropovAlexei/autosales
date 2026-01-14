from aiogram import Router, F, Bot
import logging
from aiogram.fsm.context import FSMContext
from aiogram.types import CallbackQuery, InlineKeyboardMarkup
from aiogram.utils.markdown import hbold

from api import APIClient
from keyboards import inline
from keyboards.inline import PaymentCallback, suggest_best_gateway_keyboard
from config import settings

router = Router()

async def _safe_edit_message(callback_query: CallbackQuery, text: str, reply_markup: InlineKeyboardMarkup = None, parse_mode: str = None):
    """
    Safely edits message text or caption.
    """
    if callback_query.message and callback_query.message.photo:
        return await callback_query.message.edit_caption(caption=text, reply_markup=reply_markup, parse_mode=parse_mode)
    elif callback_query.message:
        return await callback_query.message.edit_text(text, reply_markup=reply_markup, parse_mode=parse_mode)
    return None


@router.callback_query(F.data == 'balance')
async def balance_handler(callback_query: CallbackQuery, api_client: APIClient):
    try:
        user_id = callback_query.from_user.id
        response = await api_client.get_user_balance(user_id)
        if response.get("success"):
            balance = response["data"]["balance"]
            await _safe_edit_message(
                callback_query,
                f"💳 Ваш текущий баланс: {hbold(f'{balance} ₽')}",
                reply_markup=inline.balance_menu(),
                parse_mode="HTML"
            )
        else:
            await _safe_edit_message(
                callback_query,
                f"Не удалось получить баланс: {response.get('error')}",
                reply_markup=inline.main_menu(bot_type=settings.bot_type)
            )
    except Exception:
        logging.exception("An error occurred in balance_handler")
        await callback_query.message.answer("Произошла непредвиденная ошибка. Попробуйте позже.")
    await callback_query.answer()

@router.callback_query(F.data == 'deposit')
async def deposit_handler(callback_query: CallbackQuery, api_client: APIClient, state: FSMContext):
    try:
        gateways_response = await api_client.get_payment_gateways()
        settings_response = await api_client.get_public_settings()

        if gateways_response.get("success"):
            gateways = gateways_response["data"]
            public_settings = settings_response
            
            await state.update_data(gateways=gateways, public_settings=public_settings)

            await _safe_edit_message(
                callback_query,
                "💰 Выберите способ пополнения:",
                reply_markup=inline.payment_gateways_menu(gateways, public_settings, settings.payment_instructions_url)
            )
        else:
            await _safe_edit_message(
                callback_query,
                f"Не удалось загрузить способы оплаты: {gateways_response.get('error')}",
                reply_markup=inline.main_menu(bot_type=settings.bot_type)
            )
    except Exception:
        logging.exception("An error occurred in deposit_handler")
        await _safe_edit_message(callback_query, "Произошла непредвиденная ошибка. Попробуйте позже.")
    await callback_query.answer()


@router.callback_query(PaymentCallback.filter(F.action == 'select_gateway'))
async def select_gateway_handler(callback_query: CallbackQuery, callback_data: PaymentCallback, state: FSMContext):
    if callback_data.force:
        await _safe_edit_message(
            callback_query,
            "Выберите сумму для пополнения:",
            reply_markup=inline.deposit_amount_menu(gateway=callback_data.gateway)
        )
        await callback_query.answer()
        return

    data = await state.get_data()
    gateways = data.get("gateways", [])
    public_settings = data.get("public_settings", {})

    if not gateways:
        # Fallback in case state is lost
        await _safe_edit_message(
            callback_query,
            "Произошла ошибка, попробуйте начать сначала.",
            reply_markup=inline.main_menu(bot_type=settings.bot_type)
        )
        await callback_query.answer()
        return

    gateways_with_bonuses = []
    for gw in gateways:
        bonus_key = f"GATEWAY_BONUS_{gw['name']}"
        bonus_value = float(public_settings.get(bonus_key, "0"))
        gateways_with_bonuses.append(
            {
                "name": gw['name'],
                "display_name": gw['display_name'],
                "bonus": bonus_value
            }
        )

    gateways_with_bonuses.sort(key=lambda x: (-x['bonus'], x['display_name']))

    selected_gateway_name = callback_data.gateway
    best_gateway = gateways_with_bonuses[0] if gateways_with_bonuses else None
    
    selected_gateway = next((gw for gw in gateways_with_bonuses if gw['name'] == selected_gateway_name), None)

    if not selected_gateway or not best_gateway:
        # Should not happen, but as a safeguard
        await _safe_edit_message(
            callback_query,
            "Произошла ошибка при выборе платежной системы.",
            reply_markup=inline.main_menu(bot_type=settings.bot_type)
        )
        await callback_query.answer()
        return

    # If selected is not the best and the best has a better bonus
    if selected_gateway['name'] != best_gateway['name'] and best_gateway['bonus'] > selected_gateway['bonus']:
        suggestion_text = (
            f"💡 Вы выбрали {selected_gateway['display_name']} (скидка {selected_gateway['bonus']}%).\n\n"
            f"Предлагаем пополнить через {best_gateway['display_name']}, "
            f"чтобы получить скидку {best_gateway['bonus']}%!"
        )
        await _safe_edit_message(
            callback_query,
            suggestion_text,
            reply_markup=suggest_best_gateway_keyboard(selected_gateway, best_gateway)
        )
    else:
        # Proceed directly if the selected is the best or there's no better bonus
        await _safe_edit_message(
            callback_query,
            "Выберите сумму для пополнения:",
            reply_markup=inline.deposit_amount_menu(gateway=callback_data.gateway)
        )
    
    await callback_query.answer()

@router.callback_query(PaymentCallback.filter(F.action == 'select_amount'))
async def select_amount_handler(callback_query: CallbackQuery, callback_data: PaymentCallback, api_client: APIClient):
    try:
        # Use telegram_id directly
        telegram_id = callback_query.from_user.id
        amount = callback_data.amount
        gateway = callback_data.gateway

        response = await api_client.create_deposit_invoice(telegram_id, gateway, amount)

        if response.get("success"):
            invoice_data = response["data"]
            order_id = invoice_data["order_id"]
            pay_url = invoice_data.get("pay_url")
            details = invoice_data.get("details")

            sent_message = None
            if pay_url:
                sent_message = await _safe_edit_message(
                    callback_query,
                    f"✅ Ваш счет на {hbold(f'{amount} ₽')} создан.\n\nНажмите на кнопку ниже, чтобы перейти к оплате.",
                    reply_markup=inline.InlineKeyboardMarkup(inline_keyboard=[
                        [inline.InlineKeyboardButton(text="Оплатить", url=pay_url)],
                        [inline.InlineKeyboardButton(text="⬅️ Назад", callback_data="deposit")]
                    ]),
                    parse_mode="HTML"
                )
            elif details:
                requisites_text = (
                    f"Реквизиты для оплаты:\n\n"
                    f"{hbold('Банк:')} {details.get('data_bank', {}).get('name', 'N/A')}\n"
                    f"{hbold('Номер карты:')} {details.get('value', 'N/A')}\n"
                    f"{hbold('Получатель:')} {details.get('data_people', {}).get('surname', '')} {details.get('data_people', {}).get('name', '')} {details.get('data_people', {}).get('patronymic', '')}\n"
                    f"{hbold('Сумма:')} {details.get('data_mathematics', {}).get('amount_pay', 'N/A')} ₽\n\n"
                    f"После оплаты, пожалуйста, подождите. Статус платежа обновится автоматически в течение нескольких минут."
                )
                sent_message = await _safe_edit_message(
                    callback_query,
                    requisites_text,
                    reply_markup=inline.InlineKeyboardMarkup(inline_keyboard=[
                        [inline.InlineKeyboardButton(text="Перевод выполнен", callback_data=f"payment_confirm:{order_id}")],
                        [inline.InlineKeyboardButton(text="⬅️ Назад", callback_data="deposit")]
                    ]),
                    parse_mode="HTML"
                )
            else:
                await _safe_edit_message(
                    callback_query,
                    "Не удалось получить реквизиты для оплаты. Попробуйте другой способ.",
                    reply_markup=inline.deposit_amount_menu(gateway=gateway)
                )
                await callback_query.answer()
                return

            # Associate message_id with the invoice
            if sent_message:
                await api_client.set_invoice_message_id(order_id, sent_message.message_id)
        else:
            error_message = response.get('error', 'Неизвестная ошибка')
            await _safe_edit_message(
                callback_query,
                f"Не удалось создать счет: {error_message}",
                reply_markup=inline.deposit_amount_menu(gateway=gateway)
            )
    except Exception:
        logging.exception("An error occurred in select_amount_handler")
        await _safe_edit_message(callback_query, "Произошла непредвиденная ошибка. Попробуйте позже.")
    await callback_query.answer()

@router.callback_query(F.data.startswith("payment_confirm:"))
async def confirm_payment_handler(query: CallbackQuery, state: FSMContext, api_client: APIClient, bot: Bot):
    order_id = query.data.split(":")[1]
    
    response = await api_client.confirm_payment(order_id)
    
    if response and response.get("success"):
        await query.answer("Ваш платеж подтверждается, пожалуйста, подождите.", show_alert=True)
        await bot.edit_message_reply_markup(chat_id=query.message.chat.id, message_id=query.message.message_id, reply_markup=inline.back_to_main_menu_keyboard())
    else:
        error_message = response.get("error", "Произошла ошибка. Попробуйте позже.")
        await query.answer(f"Ошибка: {error_message}", show_alert=True)