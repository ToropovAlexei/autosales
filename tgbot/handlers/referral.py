from aiogram import Router, F
from aiogram.types import Message, CallbackQuery
from aiogram.fsm.context import FSMContext
from aiogram.utils.markdown import hbold

from states import ReferralState
from api import api_client
from keyboards import inline
from config import settings

router = Router()

@router.callback_query(F.data == "referral_program")
async def referral_program_handler(callback_query: CallbackQuery, state: FSMContext):
    await state.set_state(ReferralState.waiting_for_token)
    seller_info_response = await api_client.get_seller_info()
    if not seller_info_response.get("success"):
        await callback_query.message.edit_text(
            "Не удалось загрузить информацию о реферальной программе. Попробуйте позже.",
            reply_markup=inline.main_menu(
                referral_program_enabled=True,
                fallback_bot_username=settings.fallback_bot_username
            )
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
        reply_markup=inline.main_menu(
            referral_program_enabled=True,
            fallback_bot_username=settings.fallback_bot_username
        ),
        parse_mode="HTML"
    )

@router.message(ReferralState.waiting_for_token)
async def token_handler(message: Message, state: FSMContext):
    token = message.text
    user_id = message.from_user.id

    if not token or len(token.split(':')) != 2:
        await message.answer(
            "Это не похоже на токен бота. Пожалуйста, проверьте и отправьте еще раз.",
            reply_markup=inline.main_menu(
                referral_program_enabled=True,
                fallback_bot_username=settings.fallback_bot_username
            )
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
        referral_percentage = seller_data.get("referral_percentage", 0)

        if not seller_id:
            await message.answer("Не удалось определить ID продавца. Попробуйте позже.")
            await state.clear()
            return

        result = await api_client.create_referral_bot(user_id, seller_id, token)
        
        if result.get("success"):
            await message.answer(
                f"🎉 Поздравляем! Ваш реферальный бот успешно создан и скоро начнет работать.\n\n" 
                f"Все товары и категории из основного магазина теперь доступны в вашем боте. " 
                f"Вы будете получать {hbold(f'{referral_percentage}%')} от каждой покупки.",
                parse_mode="HTML"
            )
        else:
            error = result.get("error", "Произошла неизвестная ошибка.")
            if error == "Bot token is invalid":
                error_message = "😔 Токен невалидный. Пожалуйста, проверьте его и попробуйте снова."
            elif error == "Bot is already a referral bot":
                error_message = "😔 Этот бот уже используется в качестве реферального."
            else:
                error_message = f"😔 Произошла ошибка при создании бота: {error}"
            await message.answer(error_message)

    except Exception as e:
        await message.answer(f"Произошла непредвиденная ошибка: {e}")
    
    finally:
        await state.clear()