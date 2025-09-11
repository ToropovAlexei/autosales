from aiogram.fsm.state import State, StatesGroup

class CaptchaState(StatesGroup):
    waiting_for_answer = State()

class ReferralState(StatesGroup):
    waiting_for_token = State()