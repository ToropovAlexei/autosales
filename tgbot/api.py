import aiohttp
from config import settings

class APIClient:
    def __init__(self):
        self.base_url = settings.api_url
        self.headers = {
            "X-API-KEY": f"{settings.service_token}"
        }

    async def _request(self, method: str, endpoint: str, **kwargs):
        url = f"{self.base_url}{endpoint}"
        async with aiohttp.ClientSession(headers=self.headers) as session:
            async with session.request(method, url, **kwargs) as response:
                return await response.json()

    async def register_user(self, telegram_id: int):
        return await self._request("POST", "/users/register", json={"telegram_id": telegram_id})

    async def get_user_balance(self, user_id: int):
        return await self._request("GET", f"/users/{user_id}/balance")

    async def get_categories(self):
        return await self._request("GET", "/categories")

    async def get_products(self, category_id: int = None):
        endpoint = "/products"
        if category_id:
            endpoint += f"?category_ids[]={category_id}"
        return await self._request("GET", endpoint)

    async def buy_product(self, user_id: int, product_id: int):
        return await self._request("POST", "/orders/buy-from-balance", json={"user_id": user_id, "product_id": product_id, "quantity": 1})

    async def buy_external_product(self, user_id: int, provider: str, external_product_id: str):
        return await self._request("POST", "/orders/buy-from-balance", json={
            "user_id": user_id, 
            "provider": provider, 
            "external_product_id": external_product_id, 
            "quantity": 1
        })

    async def create_deposit(self, user_id: int, amount: int):
        return await self._request("POST", "/balance/deposit", json={"user_id": user_id, "amount": amount})

    async def update_user_captcha_status(self, user_id: int, status: bool):
        return await self._request("PUT", f"/users/{user_id}/captcha-status", json={"has_passed_captcha": status})

    async def get_referral_bots(self):
        return await self._request("GET", "/referrals")

    async def get_seller_info(self):
        return await self._request("GET", "/users/seller-settings")

    async def create_referral_bot(self, owner_id: int, seller_id: int, bot_token: str):
        return await self._request("POST", "/referrals", json={"owner_id": owner_id, "seller_id": seller_id, "bot_token": bot_token})

    async def get_my_referral_bots(self, telegram_id: int):
        return await self._request("GET", f"/referrals/user/{telegram_id}")

    async def set_primary_bot(self, bot_id: int, telegram_id: int):
        return await self._request("PUT", f"/referrals/{bot_id}/set-primary", json={"telegram_id": telegram_id})

    async def delete_referral_bot(self, bot_id: int, telegram_id: int):
        return await self._request("DELETE", f"/referrals/{bot_id}", json={"telegram_id": telegram_id})

api_client = APIClient()
