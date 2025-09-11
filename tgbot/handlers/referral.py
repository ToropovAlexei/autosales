from aiogram import Router, F, Bot
from aiogram.types import Message, CallbackQuery
from aiogram.fsm.context import FSMContext

from states import ReferralState
from keyboards.inline import get_main_menu
from api import api_client

router = Router()

@router.callback_query(F.data == "referral_program")
async def referral_program_handler(callback_query: CallbackQuery, state: FSMContext):
    await callback_query.message.answer(
        "Чтобы создать свой реферальный магазин, вам нужно создать собственного бота через @BotFather и прислать мне его токен."
    )
    await state.set_state(ReferralState.waiting_for_token)
    await callback_query.answer()

@router.message(ReferralState.waiting_for_token)
async def process_token(message: Message, state: FSMContext):
    token = message.text
    if not token:
        await message.answer("Пожалуйста, введите токен.")
        return

    # Basic token validation
    if len(token.split(":")) != 2:
        await message.answer("Неверный формат токена. Попробуйте еще раз.")
        return

    # Validate token with getMe
    try:
        test_bot = Bot(token=token)
        await test_bot.get_me()
    except Exception:
        await message.answer("Токен невалиден. Пожалуйста, проверьте его и попробуйте снова.")
        return
    finally:
        await test_bot.session.close()

    user_id = message.from_user.id
    # We need the internal user ID, not the telegram ID.
    # This is a shortcut, in a real app we would get this from the DB
    # based on the telegram_id.
    # For now, I will assume the telegram_id is the user_id in the bot_users table.
    user_response = await api_client.register_user(user_id)
    internal_user_id = user_response.get("data", {}).get("user", {}).get("id")

    if not internal_user_id:
        await message.answer("Не удалось найти вашего пользователя. Пожалуйста, перезапустите бота.")
        return

    # Get seller ID from seller info
    seller_info_response = await api_client.get_seller_info()
    seller_id = seller_info_response.get("data", {}).get("id")

    if not seller_id:
        await message.answer("Не удалось определить продавца. Попробуйте позже.")
        return

    response = await api_client.create_referral_bot(internal_user_id, seller_id, token)

    if response.get("status") == "success":
        await message.answer("Ваш реферальный бот успешно создан! Скоро он будет запущен.")
    else:
        await message.answer(f"Ошибка при создании бота: {response.get('message')}")

    await state.clear()
