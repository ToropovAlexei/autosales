from aiogram import Router, F
import logging
from aiogram.types import Message, CallbackQuery
from aiogram.fsm.context import FSMContext
from aiogram.utils.markdown import hbold
from aiogram.filters.callback_data import CallbackData
from aiogram.types import InlineKeyboardMarkup, InlineKeyboardButton

from states import ReferralState
from api import api_client
from keyboards import inline
from config import settings

router = Router()

class BotCallback(CallbackData, prefix="bot"):
    action: str
    bot_id: int = 0

class BotInfoCallback(CallbackData, prefix="bot_info"):
    username: str
    is_primary: str # '1' or '0'
    is_active: str  # '1' or '0'

def my_bots_keyboard(bots: list):
    buttons = []
    for bot in bots:
        status = "(Основной)" if bot.get('is_primary') else "(Активен)" if bot.get('is_active') else "(Неактивен)"
        bot_username = bot.get('bot_token').split(':')[0] # Simplified, should get from getMe
        
        is_primary_str = '1' if bot.get('is_primary') else '0'
        is_active_str = '1' if bot.get('is_active') else '0'
        info_callback_data = BotInfoCallback(
            username=bot_username,
            is_primary=is_primary_str,
            is_active=is_active_str
        ).pack()

        buttons.append([InlineKeyboardButton(text=f"@{bot_username} {status}", callback_data=info_callback_data)])
        
        action_buttons = []
        if not bot.get('is_primary'):
            action_buttons.append(InlineKeyboardButton(text="Сделать основным", callback_data=BotCallback(action="set_primary", bot_id=bot.get('id')).pack()))
        action_buttons.append(InlineKeyboardButton(text="Удалить", callback_data=BotCallback(action="delete", bot_id=bot.get('id')).pack()))
        buttons.append(action_buttons)

    if len(bots) < 3:
        buttons.append([InlineKeyboardButton(text="➕ Добавить бота", callback_data=BotCallback(action="add").pack())])
    
    buttons.append([InlineKeyboardButton(text="⬅️ Назад", callback_data="main_menu")])
    return InlineKeyboardMarkup(inline_keyboard=buttons)

async def show_my_bots(query: CallbackQuery):
    response = await api_client.get_my_referral_bots(query.from_user.id)
    if response.get("success"):
        bots = response.get("data", [])
        await query.message.edit_text(
            "Управление вашими реферальными ботами:",
            reply_markup=my_bots_keyboard(bots)
        )
    else:
        seller_info_response = await api_client.get_seller_info()
        referral_program_enabled = seller_info_response.get("data", {}).get("referral_program_enabled", False)
        await query.message.edit_text("Не удалось получить список ваших ботов. Попробуйте позже.", reply_markup=inline.main_menu(
            referral_program_enabled=referral_program_enabled,
            bot_type=settings.bot_type
        ))
    await query.answer()

@router.callback_query(BotInfoCallback.filter())
async def bot_info_handler(callback_query: CallbackQuery, callback_data: BotInfoCallback):
    primary_status = "Основной" if callback_data.is_primary == '1' else "Резервный"
    active_status = "Активен" if callback_data.is_active == '1' else "Неактивен"
    
    text = f"Бот @{callback_data.username}\nСтатус: {active_status}, {primary_status}"
    
    await callback_query.answer(text, show_alert=True)

@router.callback_query(F.data == "referral_program")
async def my_bots_handler(callback_query: CallbackQuery):
    await show_my_bots(callback_query)

@router.callback_query(BotCallback.filter(F.action == "set_primary"))
async def set_primary_handler(callback_query: CallbackQuery, callback_data: BotCallback):
    response = await api_client.set_primary_bot(callback_data.bot_id, callback_query.from_user.id)
    if response.get("success"):
        bots = response.get("data", [])
        await callback_query.message.edit_text(
            "Основной бот изменен. Управление вашими реферальными ботами:",
            reply_markup=my_bots_keyboard(bots)
        )
    else:
        await callback_query.answer("Не удалось назначить бота основным. Попробуйте позже.", show_alert=True)
    await callback_query.answer()

@router.callback_query(BotCallback.filter(F.action == "delete"))
async def delete_bot_handler(callback_query: CallbackQuery, callback_data: BotCallback):
    await api_client.delete_referral_bot(callback_data.bot_id, callback_query.from_user.id)
    await callback_query.answer("Бот удален.", show_alert=True)
    await show_my_bots(callback_query)

@router.callback_query(BotCallback.filter(F.action == "add"))
async def add_bot_handler(callback_query: CallbackQuery, state: FSMContext):
    await state.set_state(ReferralState.waiting_for_token)
    seller_info_response = await api_client.get_seller_info()
    if not seller_info_response.get("success"):
        await callback_query.message.edit_text(
            "Не удалось загрузить информацию о реферальной программе. Попробуйте позже.",
            reply_markup=inline.main_menu(bot_type=settings.bot_type)
        )
        return

    referral_percentage = seller_info_response.get("data", {}).get("referral_percentage", 0)

    await callback_query.message.edit_text(
        f"Вы можете создать свой собственный магазин-бот и получать {hbold(f'{referral_percentage}%')} с каждой продажи!\n\n" 
        "Для этого:\n" 
        "1. Создайте нового бота через @BotFather в Telegram.\n" 
        "2. Получите у него токен (набор символов вида `123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11`).\n" 
        "3. Отправьте этот токен мне в следующем сообщении.\n\n" 
        "Я жду ваш токен.",
        parse_mode="HTML"
    )
    await callback_query.answer()

@router.message(ReferralState.waiting_for_token)
async def token_handler(message: Message, state: FSMContext):
    token = message.text
    user_id = message.from_user.id

    if not token or len(token.split(':')) != 2:
        await message.answer(
            "Это не похоже на токен бота. Пожалуйста, проверьте и отправьте еще раз."
        )
        return

    try:
        seller_info_response = await api_client.get_seller_info()
        if not seller_info_response.get("success"):
            await message.answer("Не удалось получить информацию о продавце. Попробуйте позже.")
            await state.clear()
            return
        
        seller_data = seller_info_response.get("data", {})
        seller_id = seller_data.get("id")

        if not seller_id:
            await message.answer("Не удалось определить ID продавца. Попробуйте позже.")
            await state.clear()
            return

        result = await api_client.create_referral_bot(user_id, seller_id, token)
        
        if result.get("success"):
            await message.answer(
                f"🎉 Поздравляем! Ваш реферальный бот успешно создан.",
                parse_mode="HTML"
            )
            # After adding, show the list of bots again
            # How to get the query object here? We can't. We'll just send a new message with the menu.
            response = await api_client.get_my_referral_bots(message.from_user.id)
            if response.get("success"):
                bots = response.get("data", [])
                await message.answer(
                    "Управление вашими реферальными ботами:",
                    reply_markup=my_bots_keyboard(bots)
                )

        else:
            error = result.get("error", "")
            if "Bot token is invalid" in error:
                error_message = "😔 Токен невалидный. Пожалуйста, проверьте его и попробуйте снова."
            elif "already exists" in error:
                error_message = "😔 Этот бот уже используется в качестве реферального."
            elif "limit exceeded" in error:
                error_message = "😔 Вы достигли лимита в 3 бота."
            else:
                error_message = "😔 Произошла ошибка. Попробуйте позже."
            await message.answer(error_message)

    except Exception:
        logging.exception("An unexpected error occurred in token_handler")
        await message.answer("Произошла непредвиденная ошибка. Попробуйте позже.")
    
    finally:
        await state.clear()