import aiohttp
import asyncio
import math
import json
from urllib.parse import urlencode
from config import settings

class APIClient:
    def __init__(self, bot_username: str):
        self.base_url = settings.api_url
        self.bot_username = bot_username
        self.headers = {
            "X-API-KEY": f"{settings.service_token}"
        }
        self._public_settings = None

    async def _request(self, method: str, endpoint: str, **kwargs):
        url = f"{self.base_url}{endpoint}"
        async with aiohttp.ClientSession(headers=self.headers) as session:
            async with session.request(method, url, **kwargs) as response:
                return await response.json()

    async def load_public_settings(self):
        self._public_settings = await self._request("GET", "/settings/public")

    async def get_public_settings(self):
        if not self._public_settings:
            await self.load_public_settings()
        return self._public_settings.get("data", {})

    async def get_support_message(self):
        public_settings = await self.get_public_settings()
        return public_settings.get("support_message", "Что-то пошло не так, попробуйте позже.")

    async def get_welcome_message(self):
        public_settings = await self.get_public_settings()
        return public_settings.get("welcome_message", "Привет! Я бот магазина. Используйте меню ниже для навигации.")

    async def get_new_user_welcome_message(self):
        public_settings = await self.get_public_settings()
        return public_settings.get("new_user_welcome_message", "Добро пожаловать, {username}!\n\nЯ - ваш личный помощник для покупок. Здесь вы можете:\n- 🛍️ Смотреть каталог товаров\n- 💰 Пополнять баланс\n- 💳 Проверять свой счет\n\nВыберите действие в меню ниже:")

    async def get_returning_user_welcome_message(self):
        public_settings = await self.get_public_settings()
        return public_settings.get("returning_user_welcome_message", "С возвращением, {username}!\n\nЯ - ваш личный помощник для покупок. Здесь вы можете:\n- 🛍️ Смотреть каталог товаров\n- 💰 Пополнять баланс\n- 💳 Проверять свой счет\n\nВыберите действие в меню ниже:")

    async def register_user(self, telegram_id: int):
        return await self._request("POST", "/users/register", json={"telegram_id": telegram_id, "bot_name": self.bot_username})

    async def get_user(self, telegram_id: int):
        return await self._request("GET", f"/users/{telegram_id}", params={"bot_name": self.bot_username})

    async def get_user_balance(self, telegram_id: int):
        return await self._request("GET", f"/users/{telegram_id}/balance")

    async def get_categories(self):
        return await self._request("GET", "/categories")

    async def get_products_for_bot(self, category_id: int = None):
        endpoint = "/bot/products"
        params = {}
        if category_id is not None:
            params["category_id"] = category_id
        return await self._request("GET", endpoint, params=params)


    async def buy_product(self, telegram_id: int, product_id: int, referral_bot_id: int = None):
        payload = {"user_id": telegram_id, "product_id": product_id, "quantity": 1}
        if referral_bot_id:
            payload["referral_bot_id"] = referral_bot_id
        return await self._request("POST", "/orders/buy-from-balance", json=payload)

    async def buy_external_product(self, telegram_id: int, provider: str, external_product_id: str, referral_bot_id: int = None):
        payload = {
            "user_id": telegram_id, 
            "provider": provider, 
            "external_product_id": external_product_id, 
            "quantity": 1
        }
        if referral_bot_id:
            payload["referral_bot_id"] = referral_bot_id
        return await self._request("POST", "/orders/buy-from-balance", json=payload)

    async def get_payment_gateways(self):
        return await self._request("GET", "/gateways")

    async def create_deposit_invoice(self, telegram_id: int, gateway_name: str, amount: float):
        return await self._request("POST", "/deposit/invoice", json={
            "telegram_id": telegram_id,
            "gateway_name": gateway_name,
            "amount": amount
        })


    async def update_user_captcha_status(self, telegram_id: int, status: bool):
        return await self._request("PUT", f"/users/{telegram_id}/captcha-status", json={"has_passed_captcha": status})

    async def get_referral_bots(self):
        return await self._request("GET", "/referrals")


    async def create_referral_bot(self, owner_telegram_id: int, bot_token: str):
        return await self._request("POST", "/referrals", json={"owner_id": owner_telegram_id, "bot_token": bot_token})

    async def get_my_referral_bots(self, telegram_id: int):
        return await self._request("GET", f"/referrals/user/{telegram_id}")

    async def get_my_referral_stats(self, telegram_id: int):
        return await self._request("GET", f"/referrals/stats/{telegram_id}")

    async def set_primary_bot(self, bot_id: int, telegram_id: int):
        return await self._request("PUT", f"/referrals/{bot_id}/set-primary", json={"telegram_id": telegram_id})

    async def delete_referral_bot(self, bot_id: int, telegram_id: int):
        return await self._request("DELETE", f"/referrals/{bot_id}", json={"telegram_id": telegram_id})

    async def get_user_subscriptions(self, telegram_id: int):
        return await self._request("GET", f"/users/{telegram_id}/subscriptions")

    async def get_user_orders(self, telegram_id: int):
        return await self._request("GET", f"/users/{telegram_id}/orders")

    async def get_order(self, order_id: int):
        return await self._request("GET", f"/orders/{order_id}")

    async def set_invoice_message_id(self, order_id: str, message_id: int):
        return await self._request("PATCH", f"/invoices/{order_id}/message-id", json={"message_id": message_id})

    async def confirm_payment(self, order_id: str):
        return await self._request("POST", f"/bot/invoices/{order_id}/confirm")

    async def cancel_payment(self, order_id: str):
        return await self._request("POST", f"/bot/invoices/{order_id}/cancel")
