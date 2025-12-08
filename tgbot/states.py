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
    add_type = State()
    add_name = State()
    add_category = State()
    add_base_price = State()
    add_initial_stock = State()
    add_subscription_period_days = State()
    add_fulfillment_text = State()
    add_fulfillment_image = State()
    add_image = State()
    add_confirm = State()