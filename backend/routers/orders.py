

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from models import models
from db import database, db_models
from security import security

router = APIRouter()

@router.post("/buy-from-balance", response_model=models.Order)
async def buy_from_balance(
    order_data: models.OrderCreate,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    # Get user and product in one query if possible, but for clarity we do it separately
    user_result = await db.execute(select(db_models.BotUser).filter(db_models.BotUser.id == order_data.user_id))
    user = user_result.scalars().first()
    if user is None:
        raise HTTPException(status_code=404, detail="Bot user not found")

    product_result = await db.execute(select(db_models.Product).filter(db_models.Product.id == order_data.product_id))
    product = product_result.scalars().first()
    if product is None:
        raise HTTPException(status_code=404, detail="Product not found")

    if product.stock <= 0:
        raise HTTPException(status_code=400, detail="Product out of stock")

    if user.balance < product.price:
        raise HTTPException(status_code=400, detail="Insufficient balance")

    # Perform transaction
    user.balance -= product.price
    product.stock -= 1

    db_order = db_models.Order(
        user_id=order_data.user_id,
        product_id=order_data.product_id,
        amount=product.price,
        status="success"
    )
    db.add(db_order)
    await db.commit()
    await db.refresh(db_order)

    return db_order

