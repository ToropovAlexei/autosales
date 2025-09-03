
from passlib.context import CryptContext

pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")

# In-memory database
DB = {
    "users": {
        1: {
            "id": 1,
            "email": "test@example.com",
            "hashed_password": pwd_context.hash("password"),
            "is_active": True,
        }
    },
    "categories": {
        1: {"id": 1, "name": "Электроника"},
        2: {"id": 2, "name": "Одежда"},
        3: {"id": 3, "name": "Книги"},
    },
    "products": {
        1: {"id": 1, "name": "Ноутбук", "category_id": 1, "price": 120000, "stock": 15},
        2: {"id": 2, "name": "Футболка", "category_id": 2, "price": 2500, "stock": 50},
        3: {"id": 3, "name": "Война и мир", "category_id": 3, "price": 1500, "stock": 30},
        4: {"id": 4, "name": "Смартфон", "category_id": 1, "price": 80000, "stock": 25},
    },
    "bot_users": {},
    "orders": {},
}

# Helper functions to get next ID
def get_next_id(table: str) -> int:
    if not DB[table]:
        return 1
    return max(DB[table].keys()) + 1
