import asyncio
import random
from faker import Faker
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from db.database import SessionLocal, engine, Base
from db.db_models import BotUser, Category, Product, Transaction, Order, StockMovement, TransactionType, StockMovementType, User, UserRole
from security.security import get_password_hash
import datetime

fake = Faker('ru_RU')

async def seed_data(
    num_users: int = 50,
    num_categories: int = 5,
    num_products: int = 20,
    num_transactions: int = 100
):
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.drop_all)
        await conn.run_sync(Base.metadata.create_all)

    async with SessionLocal() as db:
        # Create admin user
        admin_user = User(id=1, email="test@example.com", hashed_password=get_password_hash("password"), role=UserRole.admin)
        db.add(admin_user)
        await db.commit() 
        # Create BotUsers
        users = []
        for _ in range(num_users):
            user = BotUser(
                telegram_id=fake.unique.random_number(digits=9),
                is_deleted=False,
                has_passed_captcha=True
            )
            users.append(user)
        db.add_all(users)
        await db.commit()

        # Create Categories
        categories = []
        for _ in range(num_categories):
            category = Category(name=fake.word())
            categories.append(category)
        db.add_all(categories)
        await db.commit()

        # Get IDs
        user_ids = [user.id for user in (await db.execute(select(BotUser))).scalars()]
        category_ids = [category.id for category in (await db.execute(select(Category))).scalars()]

        # Create Products
        products = []
        for _ in range(num_products):
            product = Product(
                name=fake.word(),
                price=random.uniform(100, 5000),
                category_id=random.choice(category_ids)
            )
            products.append(product)
        db.add_all(products)
        await db.commit()

        product_ids = [product.id for product in (await db.execute(select(Product))).scalars()]
        product_prices = {p.id: p.price for p in (await db.execute(select(Product))).scalars()}

        # Create initial stock for products
        stock_movements = []
        for product_id in product_ids:
            stock_movement = StockMovement(
                product_id=product_id,
                type=StockMovementType.INITIAL,
                quantity=random.randint(10, 100),
                description="Initial stock",
                created_at=datetime.datetime.now(datetime.UTC)
            )
            stock_movements.append(stock_movement)
        db.add_all(stock_movements)
        await db.commit()

        # Create Transactions and Orders
        for _ in range(num_transactions):
            user_id = random.choice(user_ids)
            # 70% chance of deposit, 30% chance of purchase
            if random.random() < 0.7:
                transaction = Transaction(
                    user_id=user_id,
                    type=TransactionType.DEPOSIT,
                    amount=random.uniform(500, 10000),
                    description="Test deposit",
                    created_at=datetime.datetime.now(datetime.UTC)
                )
                db.add(transaction)
            else:
                product_id = random.choice(product_ids)
                order = Order(
                    user_id=user_id,
                    product_id=product_id,
                    amount=product_prices[product_id],
                    status="success",
                    created_at=datetime.datetime.now(datetime.UTC)
                )
                db.add(order)
        await db.commit()

        print(f"Database seeded with {num_users} users, {num_categories} categories, {num_products} products, and {num_transactions} transactions.")

if __name__ == "__main__":
    asyncio.run(seed_data())
