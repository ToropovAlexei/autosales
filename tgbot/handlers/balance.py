from aiogram import Router, F
import logging
from aiogram.types import CallbackQuery, InlineKeyboardMarkup
from aiogram.utils.markdown import hbold

from api import APIClient
from keyboards import inline
from keyboards.inline import PaymentCallback
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
async def deposit_handler(callback_query: CallbackQuery, api_client: APIClient):
    try:
        gateways_response = await api_client.get_payment_gateways()
        settings_response = await api_client.get_public_settings()

        if gateways_response.get("success"):
            gateways = gateways_response["data"]
            await _safe_edit_message(
                callback_query,
                "💰 Выберите способ пополнения:",
                reply_markup=inline.payment_gateways_menu(gateways, settings_response, settings.payment_instructions_url)
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
async def select_gateway_handler(callback_query: CallbackQuery, callback_data: PaymentCallback):
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