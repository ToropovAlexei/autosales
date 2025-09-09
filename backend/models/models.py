import enum
import datetime
from pydantic import BaseModel
from typing import Optional

class UserRole(str, enum.Enum):
    admin = "admin"
    seller = "seller"

# Category Models
class CategoryBase(BaseModel):
    name: str

class CategoryCreate(CategoryBase):
    pass

class Category(CategoryBase):
    id: int

    class Config:
        from_attributes = True

# Product Models
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

# User Models (for panel auth)
class UserBase(BaseModel):
    email: str

class UserCreate(UserBase):
    password: str
    role: UserRole = UserRole.seller

class User(UserBase):
    id: int
    is_active: bool
    role: UserRole

    class Config:
        from_attributes = True

# Token Models
class Token(BaseModel):
    access_token: str
    token_type: str

class TokenData(BaseModel):
    email: Optional[str] = None
    role: Optional[str] = None

# Bot User Models
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

# Transaction Models
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

# Order Models
class OrderBase(BaseModel):
    user_id: int
    product_id: int
    quantity: int

class OrderCreate(OrderBase):
    pass

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

class OrderResponse(Order):
    user_telegram_id: int
    product_name: str
    created_at: datetime.datetime

# Stock Movement Models
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

# Dashboard Models
class DashboardStats(BaseModel):
    total_users: int
    users_with_purchases: int
    available_products: int

class SalesOverTime(BaseModel):
    products_sold: int
    total_revenue: float
