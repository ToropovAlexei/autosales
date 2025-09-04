
import asyncio
from sqlalchemy.ext.asyncio import async_sessionmaker

from db.database import engine, Base, SessionLocal
from db.db_models import User, Category, Product
from security.security import get_password_hash

async def init_db():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.drop_all)
        await conn.run_sync(Base.metadata.create_all)

    async with SessionLocal() as session:
        # Create default user
        db_user = await session.get(User, 1)
        if not db_user:
            session.add(User(id=1, email="test@example.com", hashed_password=get_password_hash("password"), role="admin"))

        # Create default categories
        if not (await session.get(Category, 1)):
            session.add(Category(id=1, name="Электроника"))
            session.add(Category(id=2, name="Одежда"))
            session.add(Category(id=3, name="Книги"))
        
        # Create default products
        if not (await session.get(Product, 1)):
            session.add(Product(id=1, name="Ноутбук", category_id=1, price=120000, stock=15))
            session.add(Product(id=2, name="Футболка", category_id=2, price=2500, stock=50))
            session.add(Product(id=3, name="Война и мир", category_id=3, price=1500, stock=30))

        await session.commit()

if __name__ == "__main__":
    asyncio.run(init_db())
