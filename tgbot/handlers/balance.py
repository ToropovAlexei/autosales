from aiogram import Router, F
import logging
from aiogram.types import CallbackQuery
from aiogram.utils.markdown import hbold

from api import api_client
from keyboards import inline
from keyboards.inline import PaymentCallback
from config import settings

router = Router()

@router.callback_query(F.data == 'balance')
async def balance_handler(callback_query: CallbackQuery):
    try:
        user_id = callback_query.from_user.id
        response = await api_client.get_user_balance(user_id)
        if response.get("success"):
            balance = response["data"]["balance"]
            await callback_query.message.edit_text(
                f"💳 Ваш текущий баланс: {hbold(f'{balance} ₽')}",
                reply_markup=inline.balance_menu(),
                parse_mode="HTML"
            )
        else:
            await callback_query.message.edit_text(
                f"Не удалось получить баланс: {response.get('error')}",
                reply_markup=inline.main_menu(bot_type=settings.bot_type)
            )
    except Exception:
        logging.exception("An error occurred in balance_handler")
        await callback_query.message.answer("Произошла непредвиденная ошибка. Попробуйте позже.")
    await callback_query.answer()

@router.callback_query(F.data == 'deposit')
async def deposit_handler(callback_query: CallbackQuery):
    try:
        response = await api_client.get_payment_gateways()
        if response.get("success"):
            gateways = response["data"]
            await callback_query.message.edit_text(
                "💰 Выберите способ пополнения:",
                reply_markup=inline.payment_gateways_menu(gateways, settings.payment_instructions_url)
            )
        else:
            await callback_query.message.edit_text(
                f"Не удалось загрузить способы оплаты: {response.get('error')}",
                reply_markup=inline.main_menu(bot_type=settings.bot_type)
            )
    except Exception:
        logging.exception("An error occurred in deposit_handler")
        await callback_query.message.edit_text("Произошла непредвиденная ошибка. Попробуйте позже.")
    await callback_query.answer()

@router.callback_query(PaymentCallback.filter(F.action == 'select_gateway'))
async def select_gateway_handler(callback_query: CallbackQuery, callback_data: PaymentCallback):
    await callback_query.message.edit_text(
        "Выберите сумму для пополнения:",
        reply_markup=inline.deposit_amount_menu(gateway=callback_data.gateway)
    )
    await callback_query.answer()

@router.callback_query(PaymentCallback.filter(F.action == 'select_amount'))
async def select_amount_handler(callback_query: CallbackQuery, callback_data: PaymentCallback):
    try:
        # We need the internal bot_user_id, not the telegram_id
        user_response = await api_client.get_user(callback_query.from_user.id)
        if not user_response.get("success"):
            await callback_query.message.edit_text("Ошибка: не удалось найти вашего пользователя в системе.")
            await callback_query.answer()
            return
        
        bot_user_id = user_response["data"]["id"]
        amount = callback_data.amount
        gateway = callback_data.gateway

        response = await api_client.create_deposit_invoice(bot_user_id, gateway, amount)

        if response.get("success"):
            pay_url = response["data"]["pay_url"]
            await callback_query.message.edit_text(
                f"✅ Ваш счет на {hbold(f'{amount} ₽')} создан.\n\nНажмите на кнопку ниже, чтобы перейти к оплате.",
                reply_markup=inline.InlineKeyboardMarkup(inline_keyboard=[
                    [inline.InlineKeyboardButton(text="Оплатить", url=pay_url)],
                    [inline.InlineKeyboardButton(text="⬅️ Назад", callback_data="deposit")]
                ]),
                parse_mode="HTML"
            )
        else:
            await callback_query.message.edit_text(
                f"Не удалось создать счет: {response.get('error')}",
                reply_markup=inline.deposit_amount_menu(gateway=gateway)
            )
    except Exception:
        logging.exception("An error occurred in select_amount_handler")
        await callback_query.message.edit_text("Произошла непредвиденная ошибка. Попробуйте позже.")
    await callback_query.answer()