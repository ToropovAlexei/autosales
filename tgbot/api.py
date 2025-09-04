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
                response.raise_for_status()
                return await response.json()

    async def register_user(self, telegram_id: int):
        return await self._request("POST", "/users/register", json={"telegram_id": telegram_id})

    async def get_user_balance(self, user_id: int):
        return await self._request("GET", f"/users/{user_id}/balance")

    async def get_categories(self):
        return await self._request("GET", "/categories")

    async def get_products(self, category_id: int):
        return await self._request("GET", f"/products?category_id={category_id}")

    async def buy_product(self, user_id: int, product_id: int):
        return await self._request("POST", "/orders/buy-from-balance", json={"user_id": user_id, "product_id": product_id})

    async def create_deposit(self, user_id: int, amount: int):
        return await self._request("POST", "/balance/deposit", json={"user_id": user_id, "amount": amount})

api_client = APIClient()
