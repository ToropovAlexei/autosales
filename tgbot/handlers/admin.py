from aiogram import Router, F, types, Bot
import logging
from aiogram.filters import Command
from aiogram.fsm.context import FSMContext
from aiogram.types import Message, InlineKeyboardButton, InlineKeyboardMarkup, CallbackQuery
from aiogram.filters.callback_data import CallbackData
from typing import Optional

from api import APIClient
from states import AdminLogin, ProductManagement
from keyboards.inline import main_menu, admin_menu, CategoryCallback, categories_menu, AddProductCallback
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



# --- Add Product Flow ---

class AddProductCallback(CallbackData, prefix="add_prod"):
    action: str
    value: Optional[str] = None

async def get_add_product_keyboard(
    state: FSMContext,
    back_action: Optional[str] = None,
    next_action: Optional[str] = None,
    skip_action: Optional[str] = None,
):
    buttons = []
    row = []
    data = await state.get_data()

    if back_action:
        row.append(InlineKeyboardButton(text="⬅️ Назад", callback_data=AddProductCallback(action=back_action).pack()))
    
    row.append(InlineKeyboardButton(text="❌ Отмена", callback_data="admin_main_menu"))

    if next_action and data.get(next_action.replace("set_", "")):
        row.append(InlineKeyboardButton(text="➡️ Далее", callback_data=AddProductCallback(action=next_action).pack()))
    
    if skip_action:
        row.append(InlineKeyboardButton(text="Пропустить ⏭️", callback_data=AddProductCallback(action=skip_action).pack()))
        
    buttons.append(row)
    return InlineKeyboardMarkup(inline_keyboard=buttons)


@router.callback_query(F.data == "prod_add")
async def add_product_start(callback: types.CallbackQuery, state: FSMContext):
    await state.clear()
    await state.set_state(ProductManagement.add_type)
    await state.update_data(product_creation_message_id=callback.message.message_id)
    
    keyboard = InlineKeyboardMarkup(inline_keyboard=[
        [
            InlineKeyboardButton(text="📦 Товар", callback_data=AddProductCallback(action="set_type", value="item").pack()),
            InlineKeyboardButton(text="갱 Подписка", callback_data=AddProductCallback(action="set_type", value="subscription").pack())
        ],
        [InlineKeyboardButton(text="❌ Отмена", callback_data="admin_main_menu")]
    ])
    
    await callback.message.edit_text("Выберите тип товара:", reply_markup=keyboard)
    await callback.answer()


@router.callback_query(AddProductCallback.filter(F.action == "set_type"))
async def add_product_set_type(callback: types.CallbackQuery, callback_data: AddProductCallback, state: FSMContext):
    await state.update_data(type=callback_data.value)
    await state.set_state(ProductManagement.add_name)
    
    keyboard = await get_add_product_keyboard(state, back_action="start_over")
    await callback.message.edit_text("Введите название товара:", reply_markup=keyboard)


@router.callback_query(AddProductCallback.filter(F.action == "start_over"))
async def add_product_start_over(callback: types.CallbackQuery, state: FSMContext):
    await add_product_start(callback, state)


@router.message(ProductManagement.add_name)
async def add_product_set_name(message: Message, state: FSMContext, api_client: APIClient):
    await state.update_data(name=message.text)
    await state.set_state(ProductManagement.add_category)
    await message.delete()
    
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")

    categories_response = await api_client.get_categories()
    if not categories_response.get("success"):
        await message.bot.edit_message_text("Не удалось загрузить категории.", chat_id=message.chat.id, message_id=message_id)
        return
    all_categories = categories_response["data"]
    
    keyboard = categories_menu(all_categories, 0, mode="select")
    await message.bot.edit_message_text(
        "Выберите категорию:",
        chat_id=message.chat.id,
        message_id=message_id,
        reply_markup=keyboard
    )

@router.callback_query(CategoryCallback.filter(F.action == 'view'), ProductManagement.add_category)
async def admin_navigate_categories(callback_query: CallbackQuery, callback_data: CategoryCallback, state: FSMContext, api_client: APIClient):
    category_id = callback_data.category_id
    
    categories_response = await api_client.get_categories()
    if not categories_response.get("success"):
        await callback_query.message.edit_text("Не удалось загрузить категории.")
        await callback_query.answer()
        return
    all_categories = categories_response["data"]

    if category_id == 0:  # Root level
        await callback_query.message.edit_text(
            "🛍️ Выберите категорию:",
            reply_markup=categories_menu(all_categories, 0, mode="select", category_id=0)
        )
    else:
        selected_category = find_category_by_id(all_categories, category_id)
        if not selected_category:
            await callback_query.message.edit_text("Категория не найдена.")
            await callback_query.answer()
            return

        sub_categories = selected_category.get('sub_categories', [])
        parent_id_for_back = selected_category.get('parent_id')

        reply_markup = categories_menu(sub_categories, parent_id_for_back or 0, [], category_id, mode="select")

        await callback_query.message.edit_text(
            "🛍️ Выберите категорию:",
            reply_markup=reply_markup
        )
    await callback_query.answer()

@router.callback_query(CategoryCallback.filter(F.action == 'back'), ProductManagement.add_category)
async def admin_go_back_category(callback_query: CallbackQuery, callback_data: CategoryCallback, state: FSMContext, api_client: APIClient):
    target_category_id = callback_data.category_id

    categories_response = await api_client.get_categories()
    if not categories_response.get("success"):
        await callback_query.message.edit_text("Не удалось загрузить категории.")
        await callback_query.answer()
        return
    all_categories = categories_response["data"]

    categories_to_show = []
    parent_id_for_back = 0
    if target_category_id == 0:
        categories_to_show = all_categories
    else:
        parent_id_for_back = find_parent_id(all_categories, target_category_id) or 0
        parent_category = find_category_by_id(all_categories, parent_id_for_back)
        if parent_category:
            categories_to_show = parent_category.get('sub_categories', [])
        elif parent_id_for_back == 0:
             categories_to_show = all_categories

    reply_markup = categories_menu(categories_to_show, parent_id_for_back, [], category_id=target_category_id, mode="select")

    await callback_query.message.edit_text(
        "🛍️ Выберите категорию:",
        reply_markup=reply_markup
    )
    await callback_query.answer()

# Helper function to find category by id, since it's used in catalog and admin
def find_category_by_id(categories: list, category_id: int):
    """Recursively find a category in the tree."""
    for category in categories:
        if category['id'] == category_id:
            return category
        if category.get('sub_categories'):
            found = find_category_by_id(category['sub_categories'], category_id)
            if found:
                return found
    return None

def find_parent_id(categories: list, child_id: int):
    """Recursively find the parent_id of a category."""
    for category in categories:
        if category['id'] == child_id:
            return category.get('parent_id')
        if category.get('sub_categories'):
            parent_id = find_parent_id(category['sub_categories'], child_id)
            if parent_id is not None:
                return parent_id
    return None

@router.callback_query(CategoryCallback.filter(F.action == 'select'), ProductManagement.add_category)
async def admin_select_category(callback_query: CallbackQuery, callback_data: CategoryCallback, state: FSMContext, api_client: APIClient):
    await state.update_data(category_id=callback_data.category_id)
    await state.set_state(ProductManagement.add_base_price)
    
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")

    keyboard = await get_add_product_keyboard(state, back_action="set_name")
    await callback_query.message.edit_text(
        "Введите базовую цену:",
        reply_markup=keyboard
    )
    await callback_query.answer()


@router.message(ProductManagement.add_base_price)
async def add_product_set_base_price(message: Message, state: FSMContext):
    try:
        price = float(message.text)
        await state.update_data(base_price=price)
        
        data = await state.get_data()
        product_type = data.get("type")
        
        if product_type == "item":
            await state.set_state(ProductManagement.add_initial_stock)
            next_step_text = "Введите начальное количество на складе:"
            back_action = "set_category"
        else: # subscription
            await state.set_state(ProductManagement.add_subscription_period_days)
            next_step_text = "Введите срок подписки (в днях):"
            back_action = "set_category"

        await message.delete()
        message_id = data.get("product_creation_message_id")
        keyboard = await get_add_product_keyboard(state, back_action=back_action)
        await message.bot.edit_message_text(
            next_step_text,
            chat_id=message.chat.id,
            message_id=message_id,
            reply_markup=keyboard
        )
    except ValueError:
        await message.answer("Цена должна быть числом. Попробуйте еще раз.")

@router.message(ProductManagement.add_initial_stock)
async def add_product_set_initial_stock(message: Message, state: FSMContext):
    try:
        stock = int(message.text)
        await state.update_data(initial_stock=stock)
        await state.set_state(ProductManagement.add_fulfillment_text)
        await message.delete()

        data = await state.get_data()
        message_id = data.get("product_creation_message_id")
        keyboard = await get_add_product_keyboard(
            state,
            back_action="set_base_price",
            skip_action="skip_fulfillment_text"
        )
        await message.bot.edit_message_text(
            "Введите текст для выдачи (или пропустите этот шаг):",
            chat_id=message.chat.id,
            message_id=message_id,
            reply_markup=keyboard
        )
    except ValueError:
        await message.answer("Количество должно быть целым числом. Попробуйте еще раз.")

@router.message(ProductManagement.add_subscription_period_days)
async def add_product_set_subscription_period_days(message: Message, state: FSMContext):
    try:
        days = int(message.text)
        await state.update_data(subscription_period_days=days)
        await state.set_state(ProductManagement.add_fulfillment_text)
        await message.delete()

        data = await state.get_data()
        message_id = data.get("product_creation_message_id")
        keyboard = await get_add_product_keyboard(
            state,
            back_action="set_base_price",
            skip_action="skip_fulfillment_text"
        )
        await message.bot.edit_message_text(
            "Введите текст для выдачи (или пропустите этот шаг):",
            chat_id=message.chat.id,
            message_id=message_id,
            reply_markup=keyboard
        )
    except ValueError:
        await message.answer("Срок должен быть целым числом. Попробуйте еще раз.")

@router.callback_query(AddProductCallback.filter(F.action == "skip_fulfillment_text"))
async def add_product_skip_fulfillment_text(callback: types.CallbackQuery, state: FSMContext):
    await state.update_data(fulfillment_text=None)
    await state.set_state(ProductManagement.add_fulfillment_image)

    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    back_action = "set_initial_stock" if data.get("type") == "item" else "set_subscription_period_days"
    keyboard = await get_add_product_keyboard(
        state,
        back_action=back_action,
        skip_action="skip_fulfillment_image"
    )
    await callback.message.edit_text(
        "Отправьте изображение для выдачи (или пропустите этот шаг):",
        reply_markup=keyboard
    )
    await callback.answer()

@router.message(ProductManagement.add_fulfillment_text)
async def add_product_set_fulfillment_text(message: Message, state: FSMContext):
    await state.update_data(fulfillment_text=message.text)
    await state.set_state(ProductManagement.add_fulfillment_image)
    await message.delete()

    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    back_action = "set_initial_stock" if data.get("type") == "item" else "set_subscription_period_days"
    keyboard = await get_add_product_keyboard(
        state,
        back_action=back_action,
        skip_action="skip_fulfillment_image"
    )
    await message.bot.edit_message_text(
        "Отправьте изображение для выдачи (или пропустите этот шаг):",
        chat_id=message.chat.id,
        message_id=message_id,
        reply_markup=keyboard
    )

@router.callback_query(AddProductCallback.filter(F.action == "skip_fulfillment_image"))
async def add_product_skip_fulfillment_image(callback: types.CallbackQuery, state: FSMContext):
    await state.update_data(fulfillment_image_id=None)
    await state.set_state(ProductManagement.add_image)

    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    keyboard = await get_add_product_keyboard(
        state,
        back_action="set_fulfillment_text",
        skip_action="skip_image"
    )
    await callback.message.edit_text(
        "Отправьте основное изображение товара (или пропустите):",
        reply_markup=keyboard
    )
    await callback.answer()

@router.message(ProductManagement.add_fulfillment_image, F.photo)
async def add_product_set_fulfillment_image(message: Message, state: FSMContext, api_client: APIClient):
    photo = message.photo[-1]
    
    # TODO: Proper image upload and get UUID
    # This is a placeholder until the image upload endpoint is ready
    image_response = await api_client.upload_image_from_file_id(photo.file_id)
    if not image_response or not image_response.get("success"):
        await message.answer("Не удалось загрузить изображение. Попробуйте еще раз.")
        return

    await state.update_data(fulfillment_image_id=image_response["data"]["id"])
    await state.set_state(ProductManagement.add_image)
    await message.delete()

    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    keyboard = await get_add_product_keyboard(
        state,
        back_action="set_fulfillment_text",
        skip_action="skip_image"
    )
    await message.bot.edit_message_text(
        "Отправьте основное изображение товара (или пропустите):",
        chat_id=message.chat.id,
        message_id=message_id,
        reply_markup=keyboard
    )

@router.message(ProductManagement.add_image, F.photo)
async def add_product_set_image(message: Message, state: FSMContext, api_client: APIClient):
    photo = message.photo[-1]
    
    image_response = await api_client.upload_image_from_file_id(photo.file_id)
    if not image_response or not image_response.get("success"):
        await message.answer("Не удалось загрузить изображение. Попробуйте еще раз.")
        return

    await state.update_data(image_id=image_response["data"]["id"])
    await state.set_state(ProductManagement.add_confirm)
    await message.delete()

    await show_product_confirmation(message.bot, message.chat.id, state)

@router.callback_query(AddProductCallback.filter(F.action == "skip_image"))
async def add_product_skip_image(callback: types.CallbackQuery, state: FSMContext):
    await state.update_data(image_id=None)
    await state.set_state(ProductManagement.add_confirm)
    await show_product_confirmation(callback.bot, callback.message.chat.id, state)

@router.callback_query(AddProductCallback.filter(F.action == "back_to_image"))
async def back_to_image(callback: types.CallbackQuery, state: FSMContext):
    await state.set_state(ProductManagement.add_fulfillment_image)
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    back_action = "set_initial_stock" if data.get("type") == "item" else "set_subscription_period_days"
    keyboard = await get_add_product_keyboard(
        state,
        back_action=back_action,
        skip_action="skip_fulfillment_image"
    )
    await callback.message.edit_text(
        "Отправьте изображение для выдачи (или пропустите этот шаг):",
        reply_markup=keyboard
    )
    await callback.answer()


async def show_product_confirmation(bot: Bot, chat_id: int, state: FSMContext):
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")

    text = f"""
<b>Подтвердите создание товара:</b>

<b>Тип:</b> {data.get("type")}
<b>Название:</b> {data.get("name")}
<b>Категория ID:</b> {data.get("category_id")}
<b>Цена:</b> {data.get("base_price")}
"""
    if data.get("type") == "item":
        text += f"<b>Начальный остаток:</b> {data.get('initial_stock')}\n"
    else:
        text += f"<b>Срок подписки:</b> {data.get('subscription_period_days')} дней\n"

    if data.get("fulfillment_text"):
        text += f"<b>Текст для выдачи:</b>\n{data.get('fulfillment_text')}\n"
    if data.get("fulfillment_image_id"):
        text += f"<b>Изображение для выдачи:</b> Да\n"

    text += f"<b>Основное изображение товара:</b> {'Да' if data.get('image_id') else 'Нет'}\n\n"
    text += "Все верно?"

    keyboard = InlineKeyboardMarkup(inline_keyboard=[
        [InlineKeyboardButton(text="✅ Да, создать", callback_data=AddProductCallback(action="confirm_creation").pack())],
        [InlineKeyboardButton(text="⬅️ Назад", callback_data=AddProductCallback(action="back_to_main_image").pack())],
        [InlineKeyboardButton(text="❌ Отмена", callback_data="admin_main_menu")]
    ])

    if message_id:
        await bot.edit_message_text(text, chat_id=chat_id, message_id=message_id, reply_markup=keyboard, parse_mode="HTML")

@router.callback_query(AddProductCallback.filter(F.action == "confirm_creation"))
async def add_product_confirm_creation(callback: types.CallbackQuery, state: FSMContext, api_client: APIClient):
    product_data = await state.get_data()
    admin_telegram_id = callback.from_user.id

    payload = {
        "name": product_data.get("name"),
        "category_id": product_data.get("category_id"),
        "base_price": product_data.get("base_price"),
        "type": product_data.get("type"),
        "image_id": product_data.get("image_id"),
        "fulfillment_text": product_data.get("fulfillment_text"),
        "fulfillment_image_id": product_data.get("fulfillment_image_id"),
    }
    if product_data.get("type") == "item":
        payload["initial_stock"] = product_data.get("initial_stock", 0)
    else:
        payload["subscription_period_days"] = product_data.get("subscription_period_days", 0)

    await callback.message.edit_text("Создаем товар...")
    
    response = await api_client.create_product(payload, admin_telegram_id)

    if response and response.get("status") == 201: # 201 Created
        await callback.message.edit_text("✅ Товар успешно создан!", reply_markup=admin_menu())
    else:
        error_payload = response.get("error", "Произошла неизвестная ошибка.")
        if isinstance(error_payload, dict):
            error_msg = error_payload.get("message", "Произошла неизвестная ошибка.")
        else:
            error_msg = str(error_payload)
    await state.clear()
    await callback.answer()


@router.callback_query(AddProductCallback.filter(F.action == "back_to_main_image"))
async def back_to_main_image(callback: types.CallbackQuery, state: FSMContext):
    await state.set_state(ProductManagement.add_image)
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    keyboard = await get_add_product_keyboard(
        state,
        back_action="set_fulfillment_image",
        skip_action="skip_image"
    )
    await callback.message.edit_text(
        "Отправьте основное изображение товара (или пропустите):",
        reply_markup=keyboard
    )
    await callback.answer()

@router.callback_query(F.data == "admin_main_menu")
async def back_to_admin_menu_from_add(callback: types.CallbackQuery, state: FSMContext):
    await state.clear()
    await callback.message.edit_text("Панель администратора:", reply_markup=admin_menu())
    await callback.answer()



# Placeholders for Edit and Delete
@router.callback_query(F.data.startswith("prod_edit_start"))
async def edit_product_placeholder(callback: types.CallbackQuery):
    await callback.answer("Функция редактирования в разработке", show_alert=True)


@router.callback_query(AddProductCallback.filter(F.action == "back_to_image"))
async def back_to_image(callback: types.CallbackQuery, state: FSMContext):
    await state.set_state(ProductManagement.add_image)
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    keyboard = await get_add_product_keyboard(state, back_action="set_fulfillment_type", skip_action="skip_image")
    await callback.message.edit_text(
        "Отправьте изображение для товара (или пропустите):",
        reply_markup=keyboard
    )
    await callback.answer()

@router.callback_query(AddProductCallback.filter(F.action == "set_base_price"))
async def back_to_base_price(callback: types.CallbackQuery, state: FSMContext):
    await state.set_state(ProductManagement.add_base_price)
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    keyboard = await get_add_product_keyboard(state, back_action="set_name")
    await callback.message.edit_text(
        "Введите базовую цену:",
        message_id=message_id,
        reply_markup=keyboard
    )
    await callback.answer()

@router.callback_query(AddProductCallback.filter(F.action == "set_category"))
async def back_to_category(callback: types.CallbackQuery, state: FSMContext):
    await state.set_state(ProductManagement.add_category)
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    keyboard = await get_add_product_keyboard(state, back_action="set_type")
    await callback.message.edit_text(
        "Введите ID категории:",
        message_id=message_id,
        reply_markup=keyboard
    )
    await callback.answer()

@router.callback_query(AddProductCallback.filter(F.action == "set_name"))
async def back_to_name(callback: types.CallbackQuery, state: FSMContext):
    await state.set_state(ProductManagement.add_name)
    data = await state.get_data()
    message_id = data.get("product_creation_message_id")
    keyboard = await get_add_product_keyboard(state, back_action="start_over")
    await callback.message.edit_text(
        "Введите название товара:",
        message_id=message_id,
        reply_markup=keyboard
    )
    await callback.answer()