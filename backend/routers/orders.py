from fastapi import APIRouter, Depends, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, func
import datetime

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
        user_result = await db.execute(select(db_models.BotUser).filter(
            db_models.BotUser.telegram_id == order_data.user_id,
            db_models.BotUser.is_deleted == False
        ))
        user = user_result.scalars().first()
        if user is None:
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)

        product_result = await db.execute(select(db_models.Product).filter(db_models.Product.id == order_data.product_id))
        product = product_result.scalars().first()
        if product is None:
            return error_response("Product not found", status_code=status.HTTP_404_NOT_FOUND)

        # Calculate stock
        stock_result = await db.execute(
            select(func.sum(db_models.StockMovement.quantity)).filter(db_models.StockMovement.product_id == product.id)
        )
        stock = stock_result.scalar_one_or_none() or 0

        if stock <= 0:
            return error_response("Product out of stock", status_code=status.HTTP_400_BAD_REQUEST)

        # Calculate balance
        balance_result = await db.execute(
            select(func.sum(db_models.Transaction.amount)).filter(db_models.Transaction.user_id == user.id)
        )
        balance = balance_result.scalar_one_or_none() or 0

        if balance < product.price:
            return error_response("Insufficient balance", status_code=status.HTTP_400_BAD_REQUEST)

        db_order = db_models.Order(
            user_id=user.id,
            product_id=order_data.product_id,
            amount=product.price,
            status="success"
        )
        db.add(db_order)
        await db.flush() # Flush to get the order id

        # Perform transaction
        purchase_transaction = db_models.Transaction(
            user_id=user.id,
            order_id=db_order.id,
            type=models.TransactionType.PURCHASE,
            amount=-product.price,
            description=f"Purchase of {product.name}",
            created_at=datetime.datetime.utcnow()
        )
        db.add(purchase_transaction)

        sale_movement = db_models.StockMovement(
            product_id=product.id,
            type=models.StockMovementType.SALE,
            quantity=-1,
            description=f"Sale to user {user.id}",
            created_at=datetime.datetime.utcnow()
        )
        db.add(sale_movement)

        await db.commit()
        await db.refresh(db_order)

        response_data = models.BuyResponse(
            order=db_order,
            product_name=product.name,
            product_price=product.price
        )
        
        return success_response(response_data.model_dump())
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)