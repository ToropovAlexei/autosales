
from pydantic import BaseModel
from typing import Optional

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
    stock: int

class ProductCreate(ProductBase):
    pass

class Product(ProductBase):
    id: int

    class Config:
        from_attributes = True

# User Models (for panel auth)
class UserBase(BaseModel):
    email: str

class UserCreate(UserBase):
    password: str

class User(UserBase):
    id: int
    is_active: bool

    class Config:
        from_attributes = True

# Token Models
class Token(BaseModel):
    access_token: str
    token_type: str

class TokenData(BaseModel):
    email: Optional[str] = None

# Bot User Models
class BotUserBase(BaseModel):
    telegram_id: int

class BotUserCreate(BotUserBase):
    pass

class BotUser(BotUserBase):
    id: int
    balance: float = 0

    class Config:
        from_attributes = True

# Order Models
class OrderBase(BaseModel):
    user_id: int
    product_id: int

class OrderCreate(OrderBase):
    pass

class Order(OrderBase):
    id: int
    amount: float
    status: str

    class Config:
        from_attributes = True
