import aiohttp
import asyncio
import math
import json
import logging # Added this line
from urllib.parse import urlencode
from config import settings

class APIClient:
    def __init__(self, bot_id: int):
        self.base_url = settings.api_url
        self.headers = {
            "X-API-KEY": f"{settings.service_token}",
            "X-BOT-ID": f"{bot_id}"
        }
        self._public_settings = None

    async def _request(self, method: str, endpoint: str, extra_headers: dict = None, **kwargs):
        url = f"{self.base_url}{endpoint}"
        
        headers = self.headers.copy()
        if extra_headers:
            headers.update(extra_headers)

        try:
            timeout = aiohttp.ClientTimeout(total=15)
            async with aiohttp.ClientSession(headers=headers, timeout=timeout) as session:
                async with session.request(method, url, **kwargs) as response:
                    if response.status == 204:
                        return {"status": 204, "success": True}
                    
                    if response.content_type == 'application/json':
                        json_body = await response.json()
                        # response.json() can return None if body is empty
                        return json_body if json_body is not None else {}
                    
                    return {"status": response.status}
        except aiohttp.ClientError as e:
            print(f"API request failed: {e}")
            return {"success": False, "error": {"message": "API request failed."}}

    async def load_public_settings(self):
        self._public_settings = await self._request("GET", "/settings")

    async def get_public_settings(self):
        if not self._public_settings:
            await self.load_public_settings()
        return self._public_settings.get("data", {})

    async def get_support_message(self):
        public_settings = await self.get_public_settings()
        message = public_settings.get("bot_messages_support", "Что-то пошло не так, попробуйте позже.")
        image_id = public_settings.get("bot_messages_support_image_id")
        return message, image_id

    async def get_welcome_message(self):
        public_settings = await self.get_public_settings()
        return public_settings.get("bot_messages_returning_user_welcome", "Привет! Я бот магазина. Используйте меню ниже для навигации.")

    async def get_new_user_welcome_message(self):
        public_settings = await self.get_public_settings()
        message = public_settings.get("bot_messages_new_user_welcome", """Добро пожаловать, {username}!

Я - ваш личный помощник для покупок. Здесь вы можете:
- 🛍️ Смотреть каталог товаров
- 💰 Пополнять баланс
- 💳 Проверять свой счет

Выберите действие в меню ниже:""")
        image_id = public_settings.get("bot_messages_new_user_welcome_image_id")
        return message, image_id

    async def get_returning_user_welcome_message(self):
        public_settings = await self.get_public_settings()
        message = public_settings.get("bot_messages_returning_user_welcome", """С возвращением, {username}!

Я - ваш личный помощник для покупок. Здесь вы можете:
- 🛍️ Смотреть каталог товаров
- 💰 Пополнять баланс
- 💳 Проверять свой счет

Выберите действие в меню ниже:""")
        image_id = public_settings.get("bot_messages_returning_user_welcome_image_id")
        return message, image_id

    async def register_user(self, telegram_id: int):
        return await self._request("POST", "/customers", json={"telegram_id": telegram_id})

    async def get_user(self, telegram_id: int):
        return await self._request("GET", f"/customers/{telegram_id}")
    
    async def ensure_user(self, telegram_id: int):
        user = await self.get_user(telegram_id)
        if user.get("error"):
            return await self.register_user(telegram_id)
        return user

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

    async def get_product_for_bot(self, product_id: int):
        return await self._request("GET", f"/bot/products/{product_id}")


    async def buy_product(self, telegram_id: int, product_id: int, referral_bot_id: int = None):
        payload = {"user_id": telegram_id, "product_id": product_id, "quantity": 1}
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
        return await self._request("PATCH", f"/customers/{telegram_id}", json={"has_passed_captcha": status})

    async def update_user_status(self, telegram_id: int, payload: dict):
        return await self._request("PATCH", f"/users/{telegram_id}", json=payload)

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

    async def get_my_invoices(self, telegram_id: int, page: int = 1, limit: int = 10):
        params = {"page": page, "limit": limit}
        return await self._request("GET", f"/users/{telegram_id}/invoices", params=params)

    async def get_invoice_by_id(self, invoice_id: int):
        return await self._request("GET", f"/invoices/{invoice_id}")
    
    async def get_order(self, order_id: int):
        return await self._request("GET", f"/bot/orders/{order_id}")

    async def set_invoice_message_id(self, order_id: str, message_id: int):
        return await self._request("PATCH", f"/invoices/{order_id}/message-id", json={"message_id": message_id})

    async def confirm_payment(self, order_id: str):
        return await self._request("POST", f"/bot/invoices/{order_id}/confirm")

    async def cancel_payment(self, order_id: str):
        return await self._request("POST", f"/bot/invoices/{order_id}/cancel")

    async def get_captcha(self, length: int = 6, width: int = 360, height: int = 90):
        params = {
            "length": length,
            "width": width,
            "height": height,
        }
        return await self._request("GET", "/captcha", params=params)

    async def get_bot_status(self):
        return await self._request("GET", "/can-operate")

    # --- Admin Auth ---
    async def initiate_bot_admin_auth(self, email, password):
        payload = {"email": email, "password": password}
        return await self._request("POST", "/bot/auth/initiate", json=payload)

    async def complete_bot_admin_auth(self, auth_token, tfa_code, telegram_id):
        payload = {
            "auth_token": auth_token,
            "tfa_code": tfa_code,
            "telegram_id": telegram_id
        }
        return await self._request("POST", "/bot/auth/complete", json=payload)

    # --- Admin Product Management ---
    async def create_product(self, product_data: dict, admin_telegram_id: int):
        headers = {"X-Admin-Telegram-ID": str(admin_telegram_id)}
        return await self._request("POST", "/bot/admin/products", extra_headers=headers, json=product_data)

    async def update_product(self, product_id: int, product_data: dict, admin_telegram_id: int):
        headers = {"X-Admin-Telegram-ID": str(admin_telegram_id)}
        return await self._request("PUT", f"/bot/admin/products/{product_id}", extra_headers=headers, json=product_data)

    async def delete_product(self, product_id: int, admin_telegram_id: int):
        headers = {"X-Admin-Telegram-ID": str(admin_telegram_id)}
        return await self._request("DELETE", f"/bot/admin/products/{product_id}", extra_headers=headers)

    async def get_image(self, image_path: str):
        url = f"{self.base_url.rstrip('/')}{image_path}"
        try:
            timeout = aiohttp.ClientTimeout(total=15)
            async with aiohttp.ClientSession(headers=self.headers, timeout=timeout) as session:
                async with session.get(url) as response:
                    if response.status == 200:
                        return await response.read()
                    logging.error(f"Failed to download image. Status: {response.status}, URL: {url}")
                    return None
        except aiohttp.ClientError as e:
            logging.error(f"API request for image failed: {e}")
            return None

    async def submit_receipt_link(self, order_id: str, receipt_url: str):
        return await self._request("POST", f"/invoices/{order_id}/submit-receipt", json={"receipt_url": receipt_url})
