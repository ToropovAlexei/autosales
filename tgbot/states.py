from aiogram.fsm.state import State, StatesGroup

class CaptchaState(StatesGroup):
    waiting_for_answer = State()

class ReferralState(StatesGroup):
    waiting_for_token = State()

class AdminLogin(StatesGroup):
    waiting_for_email = State()
    waiting_for_password = State()
    waiting_for_tfa = State()

class ProductManagement(StatesGroup):
    menu = State()
    add_name = State()
    add_price = State()
    add_category = State()
    add_description = State()
    add_photo = State()
    add_confirm = State()
    
    edit_select_product = State()
    edit_select_field = State()
    edit_enter_value = State()

    delete_select_product = State()
    delete_confirm = State()