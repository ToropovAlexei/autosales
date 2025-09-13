import enum
import datetime
from pydantic import BaseModel
from typing import Optional

class UserRole(str, enum.Enum):
    admin = "admin"
    seller = "seller"

# Модели Категорий
class CategoryBase(BaseModel):
    name: str

class CategoryCreate(CategoryBase):
    pass

class Category(CategoryBase):
    id: int

    class Config:
        from_attributes = True

# Модели Продуктов
class ProductBase(BaseModel):
    name: str
    category_id: int
    price: float

class ProductCreate(ProductBase):
    initial_stock: int = 0

class Product(ProductBase):
    id: int
    stock: int = 0

    class Config:
        from_attributes = True

# Модели Пользователей (для панели администратора)
class UserBase(BaseModel):
    email: str

class UserCreate(UserBase):
    password: str
    role: UserRole = UserRole.seller

class User(UserBase):
    id: int
    is_active: bool
    role: UserRole
    referral_program_enabled: bool
    referral_percentage: float

    class Config:
        from_attributes = True

class ReferralSettings(BaseModel):
    referral_program_enabled: bool
    referral_percentage: float

# Модели Токенов
class Token(BaseModel):
    access_token: str
    token_type: str

class TokenData(BaseModel):
    email: Optional[str] = None
    role: Optional[str] = None

# Модели Пользователей Бота
class BotUserBase(BaseModel):
    telegram_id: int

class BotUserCreate(BotUserBase):
    pass

class BotUser(BotUserBase):
    id: int
    is_deleted: bool = False
    has_passed_captcha: bool = False
    balance: float = 0

    class Config:
        from_attributes = True

# Модели Транзакций
class TransactionType(str, enum.Enum):
    DEPOSIT = "deposit"
    PURCHASE = "purchase"

class TransactionBase(BaseModel):
    user_id: int
    order_id: Optional[int] = None
    type: TransactionType
    amount: float
    description: Optional[str] = None

class TransactionCreate(TransactionBase):
    pass

class Transaction(TransactionBase):
    id: int
    created_at: datetime.datetime

    class Config:
        from_attributes = True

# Модели Заказов
class OrderBase(BaseModel):
    user_id: int
    product_id: int
    quantity: int

class OrderCreate(OrderBase):
    referral_bot_token: Optional[str] = None

class Order(OrderBase):
    id: int
    amount: float
    status: str
    quantity: int
    created_at: datetime.datetime

    class Config:
        from_attributes = True

class BuyResponse(BaseModel):
    order: Order
    product_name: str
    product_price: float
    balance: float

class OrderResponse(Order):
    user_telegram_id: int
    product_name: str
    created_at: datetime.datetime

# Модели Движения Склада
class StockMovementType(str, enum.Enum):
    INITIAL = "initial"
    SALE = "sale"
    RESTOCK = "restock"
    RETURN = "return"

class StockMovementBase(BaseModel):
    product_id: int
    type: StockMovementType
    quantity: int
    description: Optional[str] = None

class StockMovementCreate(StockMovementBase):
    pass

class StockMovement(StockMovementBase):
    id: int
    created_at: datetime.datetime

    class Config:
        from_attributes = True

# Модели для Дашборда
class DashboardStats(BaseModel):
    total_users: int
    users_with_purchases: int
    available_products: int

class SalesOverTime(BaseModel):
    products_sold: int
    total_revenue: float

# Модели Реферальных Ботов
class ReferralBotBase(BaseModel):
    owner_id: int
    seller_id: int
    bot_token: str

class ReferralBotCreate(ReferralBotBase):
    pass

class ReferralBot(ReferralBotBase):
    id: int
    is_active: bool
    created_at: datetime.datetime

    class Config:
        from_attributes = True

class ReferralBotAdminInfo(ReferralBot):
    owner_telegram_id: int
    turnover: float = 0
    accruals: float = 0

# Модели Реферальных Транзакций
class RefTransactionBase(BaseModel):
    ref_owner_id: int
    seller_id: int
    order_id: int
    amount: float
    ref_share: float

class RefTransactionCreate(RefTransactionBase):
    pass

class RefTransaction(RefTransactionBase):
    id: int
    created_at: datetime.datetime

    class Config:
        from_attributes = True
