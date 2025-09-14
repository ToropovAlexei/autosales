import enum
import datetime
from sqlalchemy import BIGINT, Boolean, Column, Integer, String, Float, ForeignKey, Enum, DateTime
from sqlalchemy.orm import relationship

from .database import Base

class UserRole(str, enum.Enum):
    admin = "admin"
    seller = "seller"

class User(Base):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True, index=True)
    email = Column(String, unique=True, index=True)
    hashed_password = Column(String)
    is_active = Column(Boolean, default=True)
    role = Column(Enum(UserRole), default=UserRole.seller, nullable=False)
    referral_program_enabled = Column(Boolean, default=False)
    referral_percentage = Column(Float, default=0.0)

class Category(Base):
    __tablename__ = "categories"

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, index=True)

    products = relationship("Product", back_populates="category")

class Product(Base):
    __tablename__ = "products"

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, index=True)
    price = Column(Float)
    category_id = Column(Integer, ForeignKey("categories.id"))

    category = relationship("Category", back_populates="products")

class BotUser(Base):
    __tablename__ = "bot_users"

    id = Column(Integer, primary_key=True, index=True)
    telegram_id = Column(BIGINT, unique=True, index=True)
    is_deleted = Column(Boolean, default=False)
    has_passed_captcha = Column(Boolean, default=False)

class TransactionType(str, enum.Enum):
    DEPOSIT = "deposit"
    PURCHASE = "purchase"

class Transaction(Base):
    __tablename__ = "transactions"

    id = Column(Integer, primary_key=True, index=True)
    user_id = Column(Integer, ForeignKey("bot_users.id"))
    order_id = Column(Integer, ForeignKey("orders.id"), nullable=True)
    type = Column(Enum(TransactionType), nullable=False)
    amount = Column(Float, nullable=False)
    created_at = Column(DateTime(timezone=True), nullable=False)
    description = Column(String)

class Order(Base):
    __tablename__ = "orders"

    id = Column(Integer, primary_key=True, index=True)
    user_id = Column(Integer, ForeignKey("bot_users.id"))
    product_id = Column(Integer, ForeignKey("products.id"))
    quantity = Column(Integer, default=1)
    amount = Column(Float)
    status = Column(String)
    created_at = Column(DateTime(timezone=True), nullable=False, default=datetime.datetime.now(datetime.UTC))

class StockMovementType(str, enum.Enum):
    INITIAL = "initial"
    SALE = "sale"
    RESTOCK = "restock"
    RETURN = "return"

class StockMovement(Base):
    __tablename__ = "stock_movements"

    id = Column(Integer, primary_key=True, index=True)
    order_id = Column(Integer, ForeignKey("orders.id"), nullable=True)
    product_id = Column(Integer, ForeignKey("products.id"))
    type = Column(Enum(StockMovementType), nullable=False)
    quantity = Column(Integer, nullable=False)
    created_at = Column(DateTime(timezone=True), nullable=False)
    description = Column(String)

class ReferralBot(Base):
    __tablename__ = "referral_bots"

    id = Column(Integer, primary_key=True, index=True)
    owner_id = Column(Integer, ForeignKey("bot_users.id"))
    seller_id = Column(Integer, ForeignKey("users.id"))
    bot_token = Column(String, unique=True)
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime(timezone=True), nullable=False, default=datetime.datetime.now(datetime.UTC))

class RefTransaction(Base):
    __tablename__ = "ref_transactions"

    id = Column(Integer, primary_key=True, index=True)
    ref_owner_id = Column(Integer, ForeignKey("bot_users.id"))
    seller_id = Column(Integer, ForeignKey("users.id"))
    order_id = Column(Integer, ForeignKey("orders.id"))
    amount = Column(Float, nullable=False)
    ref_share = Column(Float, nullable=False)
    created_at = Column(DateTime(timezone=True), nullable=False, default=datetime.datetime.now(datetime.UTC))