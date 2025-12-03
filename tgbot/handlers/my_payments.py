# tgbot/handlers/my_payments.py
import logging
import json # New import
from aiogram import Router, F, types
from aiogram.exceptions import TelegramBadRequest
from aiogram.utils.markdown import hbold # New import

from api import APIClient
from keyboards.inline import back_to_main_menu_keyboard

router = Router()

def format_invoice_info(invoice):
    """Formats a single invoice line."""
    date = invoice['CreatedAt'][:10] # Extract YYYY-MM-DD
    return f"Платеж #{invoice['ID']} на {invoice['Amount']} RUB от {date}"

@router.callback_query(F.data == "my_payments")
async def show_payments_handler(callback: types.CallbackQuery, api_client: APIClient):
    """
    Shows the user's payment history, divided into active and completed.
    """
    await callback.answer()
    telegram_id = callback.from_user.id
    
    try:
        response = await api_client.get_my_invoices(telegram_id, page=1, limit=20)
        if not response.get("success"):
            await callback.message.edit_text(
                "Не удалось загрузить историю платежей. Попробуйте позже.",
                reply_markup=back_to_main_menu_keyboard()
            )
            return

        invoices = response.get("data", {}).get("data", [])
        
        active_invoices = [inv for inv in invoices if inv['Status'] == 'pending']
        completed_invoices = [inv for inv in invoices if inv['Status'] == 'completed']

        text = "<b>🧾 Мои платежи</b>\n\n"
        buttons = []

        if active_invoices:
            text += "<u>Активные платежи:</u>\n"
            for inv in active_invoices:
                text += f"• {format_invoice_info(inv)}\n"
                buttons.append([types.InlineKeyboardButton(
                    text=f"Посмотреть счет #{inv['ID']}",
                    callback_data=f"view_invoice_{inv['ID']}"
                )])
            text += "\n"
        else:
            text += "У вас нет активных счетов для оплаты.\n\n"

        if completed_invoices:
            text += "<u>История операций:</u>\n"
            for inv in completed_invoices[:5]: # Show last 5
                text += f"• {format_invoice_info(inv)}\n"
        
        # TODO: Add pagination later if needed. For now, showing top 20.

        buttons.append([types.InlineKeyboardButton(text="⬅️ Назад", callback_data="main_menu")])
        
        await callback.message.edit_text(
            text,
            parse_mode="HTML",
            reply_markup=types.InlineKeyboardMarkup(inline_keyboard=buttons)
        )

    except Exception as e:
        logging.error(f"Error fetching user payments: {e}")
        await callback.message.edit_text(
            "Произошла ошибка при загрузке ваших платежей.",
            reply_markup=back_to_main_menu_keyboard()
        )

@router.callback_query(F.data.startswith("view_invoice_"))
async def view_invoice_handler(callback: types.CallbackQuery, api_client: APIClient):
    """
    Reconstructs and displays the original invoice message to the user.
    """
    await callback.answer("Загружаю счет...")
    invoice_id_str = callback.data.split("_")[2]
    
    try:
        response = await api_client.get_invoice_by_id(int(invoice_id_str))
        if not response.get("success"):
            raise Exception("Failed to fetch invoice details")

        invoice = response.get("data", {})
        
        pay_url = invoice.get("pay_url")
        payment_details_json = invoice.get("payment_details") # This is a JSON string from Go backend
        
        text = ""
        buttons = []

        if pay_url:
            bolded_amount = hbold(f"{invoice['Amount']} RUB")
            text = (
                f"✅ Ваш счет #{invoice['ID']} на {bolded_amount} создан.\n\n"
                f"Нажмите на кнопку ниже, чтобы перейти к оплате."
            )
            buttons.append([types.InlineKeyboardButton(text="Оплатить", url=pay_url)])
        elif payment_details_json:
            details = payment_details_json # It's already a dict
            text = (
                f"Реквизиты для оплаты счета #{invoice['ID']}:\n\n"
                f"{hbold('Банк:')} {details.get('data_bank', {}).get('name', 'N/A')}\n"
                f"{hbold('Номер карты:')} {details.get('value', 'N/A')}\n"
                f"{hbold('Получатель:')} {details.get('data_people', {}).get('surname', '')} {details.get('data_people', {}).get('name', '')} {details.get('data_people', {}).get('patronymic', '')}\n"
                f"{hbold('Сумма:')} {details.get('data_mathematics', {}).get('amount_pay', 'N/A')} ₽\n\n"
                f"После оплаты, пожалуйста, подождите. Статус платежа обновится автоматически в течение нескольких минут."
            )
        else:
            text = "Не удалось получить реквизиты для оплаты. Пожалуйста, обратитесь в поддержку."

        buttons.append([types.InlineKeyboardButton(text="⬅️ Назад к платежам", callback_data="my_payments")])
        
        await callback.message.edit_text(
            text,
            parse_mode="HTML",
            reply_markup=types.InlineKeyboardMarkup(inline_keyboard=buttons)
        )

    except Exception as e:
        logging.error(f"Error in view_invoice_handler: {e}")
        await callback.message.edit_text("Произошла непредвиденная ошибка.", reply_markup=back_to_main_menu_keyboard())
