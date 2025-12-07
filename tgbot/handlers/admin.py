from aiogram import Router, F, types, Bot
import logging
from aiogram.filters import Command
from aiogram.fsm.context import FSMContext
from aiogram.types import Message, InlineKeyboardButton, InlineKeyboardMarkup

from api import APIClient
from states import AdminLogin, ProductManagement
from keyboards.inline import main_menu, admin_menu
from config import settings

router = Router()

# --- Helper Functions ---

async def _return_to_main_menu(bot: Bot, chat_id: int, message_id: int, api_client: APIClient, state: FSMContext, is_admin: bool = False):
    """Clears state and edits the message to show the main menu."""
    if not is_admin:
        await state.clear()
    
    try:
        seller_info = await api_client.get_public_settings()
        referral_enabled = seller_info.get("referral_program_enabled", False)
        
        keyboard = main_menu(
            referral_program_enabled=referral_enabled,
            bot_type=settings.bot_type,
            is_admin=is_admin
        )
        
        await bot.edit_message_text(
            "Главное меню",
            chat_id=chat_id,
            message_id=message_id,
            reply_markup=keyboard
        )
    except Exception as e:
        # Fallback in case of API error or other issues
        await bot.edit_message_text(
            "Отменено. Нажмите /start, чтобы вернуться в меню.",
            chat_id=chat_id,
            message_id=message_id
        )


@router.callback_query(F.data == "admin_panel")
async def show_admin_panel(callback: types.CallbackQuery, state: FSMContext, api_client: APIClient):
    await state.set_state(ProductManagement.menu)
    await callback.message.edit_text(
        "👑 Панель администратора",
        reply_markup=admin_menu()
    )
    await callback.answer()

@router.callback_query(F.data == "admin_login_cancel")
async def cancel_login_handler(callback: types.CallbackQuery, state: FSMContext, api_client: APIClient):
    data = await state.get_data()
    login_message_id = data.get("login_message_id")
    if login_message_id:
        await _return_to_main_menu(
            bot=callback.bot,
            chat_id=callback.message.chat.id,
            message_id=login_message_id,
            api_client=api_client,
            state=state
        )
    else:
        await callback.message.edit_text("Вход отменен. Нажмите /start для возврата в меню.")
    
    await state.clear()
    await callback.answer()

@router.message(Command("admin"))
async def cmd_admin(message: Message, state: FSMContext, api_client: APIClient):
    await message.delete()

    await state.set_state(AdminLogin.waiting_for_email)
    
    data = await state.get_data()
    main_menu_id = data.get("main_menu_id")

    prompt_text = "Добро пожаловать в панель администратора.\n\nПожалуйста, введите ваш email:"
    keyboard = InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="❌ Отмена", callback_data="admin_login_cancel")]])

    if main_menu_id:
        try:
            await message.bot.edit_message_text(
                prompt_text,
                chat_id=message.chat.id,
                message_id=main_menu_id,
                reply_markup=keyboard
            )
            await state.update_data(login_message_id=main_menu_id)
        except Exception: # If edit fails, send a new message
            sent_message = await message.answer(prompt_text, reply_markup=keyboard)
            await state.update_data(login_message_id=sent_message.message_id)
    else:
        sent_message = await message.answer(prompt_text, reply_markup=keyboard)
        await state.update_data(login_message_id=sent_message.message_id)


@router.message(AdminLogin.waiting_for_email)
async def process_email(message: Message, state: FSMContext):
    await message.delete()
    await state.update_data(email=message.text)
    await state.set_state(AdminLogin.waiting_for_password)
    
    data = await state.get_data()
    login_message_id = data.get("login_message_id")

    if login_message_id:
        await message.bot.edit_message_text(
            "Введите ваш пароль:",
            chat_id=message.chat.id,
            message_id=login_message_id,
            reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="❌ Отмена", callback_data="admin_login_cancel")]])
        )

@router.message(AdminLogin.waiting_for_password)
async def process_password(message: Message, state: FSMContext, api_client: APIClient):
    await message.delete()
    data = await state.get_data()
    email = data.get("email")
    password = message.text
    login_message_id = data.get("login_message_id")

    if login_message_id:
        await message.bot.edit_message_text(
            "Проверка данных...",
            chat_id=message.chat.id,
            message_id=login_message_id,
            reply_markup=None
        )

    response = await api_client.initiate_bot_admin_auth(email, password)
    logging.info(f"Auth response: {response}")

    if "data" in response and "auth_token" in response["data"]:
        await state.update_data(auth_token=response["data"]["auth_token"])
        await state.set_state(AdminLogin.waiting_for_tfa)
        if login_message_id:
            await message.bot.edit_message_text(
                "Пароль принят. Введите код двухфакторной аутентификации (2FA):",
                chat_id=message.chat.id,
                message_id=login_message_id,
                reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="❌ Отмена", callback_data="admin_login_cancel")]])
            )
    else:
        error_payload = response.get("error")
        error_msg = "Неверный email или пароль."
        if isinstance(error_payload, dict):
            error_msg = error_payload.get("message", error_msg)
        elif isinstance(error_payload, str):
            error_msg = error_payload

        if login_message_id:
            retry_keyboard = InlineKeyboardMarkup(inline_keyboard=[
                [InlineKeyboardButton(text="Попробовать снова", callback_data="admin_login_retry")],
                [InlineKeyboardButton(text="❌ Отмена", callback_data="admin_login_cancel")]
            ])
            await message.bot.edit_message_text(
                f"❌ Ошибка входа: {error_msg}",
                chat_id=message.chat.id,
                message_id=login_message_id,
                reply_markup=retry_keyboard
            )


@router.callback_query(F.data == "admin_login_retry")
async def retry_login_handler(callback: types.CallbackQuery, state: FSMContext):
    await state.set_state(AdminLogin.waiting_for_email)
    await callback.message.edit_text(
        "Пожалуйста, введите ваш email:",
        reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="❌ Отмена", callback_data="admin_login_cancel")]])
    )
    await callback.answer()


@router.message(AdminLogin.waiting_for_tfa)
async def process_tfa(message: Message, state: FSMContext, api_client: APIClient):
    await message.delete()
    data = await state.get_data()
    auth_token = data.get("auth_token")
    tfa_code = message.text
    telegram_id = message.from_user.id
    login_message_id = data.get("login_message_id")
    edit_target_id = login_message_id or message.message_id

    response = await api_client.complete_bot_admin_auth(auth_token, tfa_code, telegram_id)
    
    if response and response.get("success"):
        await state.clear() 
        await state.set_data({'is_admin': True})
        await state.set_state(ProductManagement.menu)
        
        await message.bot.edit_message_text(
            "✅ Авторизация пройдена успешно!",
            chat_id=message.chat.id,
            message_id=edit_target_id,
            reply_markup=admin_menu()
        )
    else:
        error_payload = response.get("error")
        error_msg = "Неверный код 2FA."
        if isinstance(error_payload, dict):
            error_msg = error_payload.get("message", error_msg)
        elif isinstance(error_payload, str):
            error_msg = error_payload
        
        # Edit the message to show the error temporarily, then revert to main menu
        await message.bot.edit_message_text(
            f"❌ Ошибка входа: {error_msg}\n\nВозврат в главное меню...",
            chat_id=message.chat.id,
            message_id=edit_target_id
        )
        # Revert to main menu
        await _return_to_main_menu(
            bot=message.bot,
            chat_id=message.chat.id,
            message_id=edit_target_id,
            api_client=api_client,
            state=state
        )

@router.callback_query(F.data == "admin_main_menu")
async def back_to_admin_menu(callback: types.CallbackQuery, state: FSMContext):
    await callback.message.edit_text("Панель администратора:", reply_markup=admin_menu())
    await state.set_state(ProductManagement.menu)
    await callback.answer()



# --- Product Management Handlers ---

# --- Add Product Flow ---

def get_product_creation_keyboard(back_callback: str = None):
    buttons = []
    if back_callback:
        buttons.append(InlineKeyboardButton(text="⬅️ Назад", callback_data=back_callback))
    buttons.append(InlineKeyboardButton(text="❌ Отмена", callback_data="admin_main_menu"))
    return InlineKeyboardMarkup(inline_keyboard=[buttons])

@router.callback_query(F.data == "prod_add", ProductManagement.menu)
async def add_product_start(callback: types.CallbackQuery, state: FSMContext):
    await state.set_state(ProductManagement.add_name)
    await state.update_data(product_creation_message_id=callback.message.message_id)
    await callback.message.edit_text(
        "Введите название нового товара:",
        reply_markup=get_product_creation_keyboard()
    )
    await callback.answer()

@router.message(ProductManagement.add_name)
async def add_product_name(message: Message, state: FSMContext):
    await state.update_data(name=message.text)
    await state.set_state(ProductManagement.add_price)
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    await message.delete()
    if message_id:
        await message.bot.edit_message_text(
            "Отлично. Теперь введите цену товара (только число, например, 1500 или 99.99):",
            chat_id=message.chat.id,
            message_id=message_id,
            reply_markup=get_product_creation_keyboard(back_callback="prod_add_back_to_name")
        )

@router.callback_query(F.data == "prod_add_back_to_name", ProductManagement.add_price)
async def back_to_add_product_name(callback: types.CallbackQuery, state: FSMContext):
    await state.set_state(ProductManagement.add_name)
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    if message_id:
        await callback.message.edit_text(
            "Введите название нового товара:",
            reply_markup=get_product_creation_keyboard()
        )
    await callback.answer()

@router.message(ProductManagement.add_price)
async def add_product_price(message: Message, state: FSMContext):
    try:
        price = float(message.text)
        await state.update_data(price=price)
        await state.set_state(ProductManagement.add_description)
        data = await state.get_data()
        message_id = data.get("product_creation_message_id")
        await message.delete()
        if message_id:
            await message.bot.edit_message_text(
                "Цена принята. Введите описание товара:",
                chat_id=message.chat.id,
                message_id=message_id,
                reply_markup=get_product_creation_keyboard(back_callback="prod_add_back_to_price")
            )
    except ValueError:
        await message.answer("Это не похоже на число. Пожалуйста, введите цену в формате 1500 или 99.99.")

@router.callback_query(F.data == "prod_add_back_to_price", ProductManagement.add_description)
async def back_to_add_product_price(callback: types.CallbackQuery, state: FSMContext):
    await state.set_state(ProductManagement.add_price)
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    if message_id:
        await callback.message.edit_text(
            "Отлично. Теперь введите цену товара (только число, например, 1500 или 99.99):",
            reply_markup=get_product_creation_keyboard(back_callback="prod_add_back_to_name")
        )
    await callback.answer()

@router.message(ProductManagement.add_description)
async def add_product_description(message: Message, state: FSMContext, api_client: APIClient):
    await state.update_data(description=message.text)
    
    product_data = await state.get_data()
    
    categories_response = await api_client.get_categories()
    if not categories_response.get("data"):
        await message.answer("Не удалось найти категории. Невозможно создать товар.")
        await state.set_state(ProductManagement.menu)
        return
        
    first_category_id = categories_response["data"][0]["id"]
    await state.update_data(category_id=first_category_id)
    
    product_data = await state.get_data()
    message_id = product_data.get("product_creation_message_id")
    await message.delete()

    text = (
        f"<b>Подтвердите создание товара:</b>\n\n"
        f"<b>Название:</b> {product_data.get('name')}\n"
        f"<b>Цена:</b> {product_data.get('price')}\n"
        f"<b>Описание:</b> {product_data.get('description')}\n"
        f"<b>Категория ID:</b> {first_category_id} (Временно)\n\n"
        f"Все верно?"
    )
    
    confirm_keyboard = InlineKeyboardMarkup(inline_keyboard=[
        [InlineKeyboardButton(text="✅ Да, создать", callback_data="prod_add_confirm")],
        [InlineKeyboardButton(text="⬅️ Назад", callback_data="prod_add_back_to_description")],
        [InlineKeyboardButton(text="❌ Нет, отмена", callback_data="admin_main_menu")]
    ])
    
    await state.set_state(ProductManagement.add_confirm)
    if message_id:
        await message.bot.edit_message_text(
            text,
            chat_id=message.chat.id,
            message_id=message_id,
            reply_markup=confirm_keyboard, 
            parse_mode="HTML"
        )

@router.callback_query(F.data == "prod_add_back_to_description", ProductManagement.add_confirm)
async def back_to_add_product_description(callback: types.CallbackQuery, state: FSMContext):
    await state.set_state(ProductManagement.add_description)
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    if message_id:
        await callback.message.edit_text(
            "Цена принята. Введите описание товара:",
            reply_markup=get_product_creation_keyboard(back_callback="prod_add_back_to_price")
        )
    await callback.answer()

@router.callback_query(F.data == "prod_add_confirm", ProductManagement.add_confirm)
async def add_product_confirm(callback: types.CallbackQuery, state: FSMContext, api_client: APIClient):
    product_data = await state.get_data()
    admin_telegram_id = callback.from_user.id

    payload = {
        "name": product_data.get("name"),
        "price": product_data.get("price"),
        "category_id": product_data.get("category_id"),
        "details": product_data.get("description")
    }

    await callback.message.edit_text("Создаем товар...")
    
    response = await api_client.create_product(payload, admin_telegram_id)

    if response.get("status") == 201: # 201 Created
        await callback.message.edit_text("✅ Товар успешно создан!", reply_markup=await get_admin_menu())
    else:
        error_payload = response.get("error", "Произошла неизвестная ошибка.")
        if isinstance(error_payload, dict):
            error_msg = error_payload.get("message", "Произошла неизвестная ошибка.")
        else:
            error_msg = str(error_payload)
        await callback.message.edit_text(f"❌ Не удалось создать товар.\nОшибка: {error_msg}", reply_markup=await get_admin_menu())
        
    await state.set_state(ProductManagement.menu)
    await callback.answer()

# Placeholders for Edit and Delete
@router.callback_query(F.data.startswith("prod_edit_start"))
async def edit_product_placeholder(callback: types.CallbackQuery):
    await callback.answer("Функция редактирования в разработке", show_alert=True)

@router.callback_query(F.data.startswith("prod_delete_start"))
async def delete_product_placeholder(callback: types.CallbackQuery):
    await callback.answer("Функция удаления в разработке", show_alert=True)