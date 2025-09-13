from aiogram import Router, F
from aiogram.types import CallbackQuery
from aiogram.utils.markdown import hbold

from api import api_client
from keyboards import inline
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
                reply_markup=inline.deposit_menu(),
                parse_mode="HTML"
            )
        else:
            seller_info_response = await api_client.get_seller_info()
            referral_program_enabled = seller_info_response.get("data", {}).get("referral_program_enabled", False)
            await callback_query.message.edit_text(
                f"Не удалось получить баланс: {response.get('error')}",
                reply_markup=inline.main_menu(
                    referral_program_enabled=referral_program_enabled,
                    fallback_bot_username=settings.fallback_bot_username
                )
            )
    except Exception as e:
        await callback_query.message.answer(f"Произошла ошибка: {e}")
    await callback_query.answer()

@router.callback_query(F.data == 'deposit')
async def deposit_handler(callback_query: CallbackQuery):
    await callback_query.message.edit_text(
        "💰 Выберите сумму для пополнения баланса:", 
        reply_markup=inline.deposit_menu()
    )
    await callback_query.answer()

@router.callback_query(F.data.startswith('deposit_'))
async def deposit_amount_handler(callback_query: CallbackQuery):
    amount = int(callback_query.data.split('_')[1])
    user_id = callback_query.from_user.id
    try:
        seller_info_response = await api_client.get_seller_info()
        referral_program_enabled = seller_info_response.get("data", {}).get("referral_program_enabled", False)

        response = await api_client.create_deposit(user_id, amount)
        if response.get("success"):
            await callback_query.message.edit_text(
                f"✅ Заявка на пополнение на {hbold(f'{amount} ₽')} успешно создана.\n\n" 
                f"В реальном приложении здесь была бы ссылка на оплату.",
                reply_markup=inline.main_menu(
                    referral_program_enabled=referral_program_enabled,
                    fallback_bot_username=settings.fallback_bot_username
                ),
                parse_mode="HTML"
            )
        else:
            await callback_query.message.edit_text(
                f"Не удалось создать заявку: {response.get('error')}",
                reply_markup=inline.main_menu(
                    referral_program_enabled=referral_program_enabled,
                    fallback_bot_username=settings.fallback_bot_username
                )
            )
    except Exception as e:
        await callback_query.message.edit_text(f"Произошла ошибка: {e}")
    await callback_query.answer()