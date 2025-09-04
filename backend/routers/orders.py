from fastapi import APIRouter, Depends, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from models import models
from db import database, db_models
from security import security
from core.responses import success_response, error_response

router = APIRouter()

@router.post("/buy-from-balance")
async def buy_from_balance(
    order_data: models.OrderCreate,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.verify_service_token)
):
    try:
        user_result = await db.execute(select(db_models.BotUser).filter(db_models.BotUser.telegram_id == order_data.user_id))
        user = user_result.scalars().first()
        if user is None:
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)

        product_result = await db.execute(select(db_models.Product).filter(db_models.Product.id == order_data.product_id))
        product = product_result.scalars().first()
        if product is None:
            return error_response("Product not found", status_code=status.HTTP_404_NOT_FOUND)

        if product.stock <= 0:
            return error_response("Product out of stock", status_code=status.HTTP_400_BAD_REQUEST)

        if user.balance < product.price:
            return error_response("Insufficient balance", status_code=status.HTTP_400_BAD_REQUEST)

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
        await db.refresh(user)
        await db.refresh(product)

        response_data = models.BuyResponse(
            order=db_order,
            balance=user.balance,
            product_name=product.name,
            product_price=product.price
        )
        
        return success_response(response_data.model_dump())
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)