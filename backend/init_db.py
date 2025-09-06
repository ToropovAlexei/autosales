import asyncio
from sqlalchemy.ext.asyncio import async_sessionmaker

from db.database import engine, Base, SessionLocal
from db.db_models import User, Category, Product, UserRole, StockMovement, StockMovementType
from security.security import get_password_hash
import datetime

async def init_db():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.drop_all)
        await conn.run_sync(Base.metadata.create_all)

    async with SessionLocal() as session:
        # Create default user
        db_user = await session.get(User, 1)
        if not db_user:
            session.add(User(id=1, email="test@example.com", hashed_password=get_password_hash("password"), role=UserRole.admin))

        # Create default categories
        if not (await session.get(Category, 1)):
            session.add(Category(id=1, name="Электроника"))
            session.add(Category(id=2, name="Одежда"))
            session.add(Category(id=3, name="Книги"))
        
        # Create default products
        if not (await session.get(Product, 1)):
            product1 = Product(id=1, name="Ноутбук", category_id=1, price=120000)
            product2 = Product(id=2, name="Футболка", category_id=2, price=2500)
            product3 = Product(id=3, name="Война и мир", category_id=3, price=1500)
            session.add_all([product1, product2, product3])
            await session.flush()

            session.add(StockMovement(product_id=1, type=StockMovementType.INITIAL, quantity=15, created_at=datetime.datetime.utcnow(), description="Initial stock"))
            session.add(StockMovement(product_id=2, type=StockMovementType.INITIAL, quantity=50, created_at=datetime.datetime.utcnow(), description="Initial stock"))
            session.add(StockMovement(product_id=3, type=StockMovementType.INITIAL, quantity=30, created_at=datetime.datetime.utcnow(), description="Initial stock"))

        await session.commit()

if __name__ == "__main__":
    asyncio.run(init_db())