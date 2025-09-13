from typing import List
from fastapi import APIRouter, Depends, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, func
import datetime
import logging

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
            db_models.BotUser.is_deleted.is_(False)
        ))
        user = user_result.scalars().first()
        if user is None:
            return error_response("Bot user not found", status_code=status.HTTP_404_NOT_FOUND)

        product_result = await db.execute(select(db_models.Product).filter(db_models.Product.id == order_data.product_id))
        product = product_result.scalars().first()
        if product is None:
            return error_response("Product not found", status_code=status.HTTP_404_NOT_FOUND)

        product_name = product.name
        product_price = product.price

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

        order_amount = product_price * order_data.quantity
        if balance < order_amount:
            return error_response("Insufficient balance", status_code=status.HTTP_400_BAD_REQUEST)

        db_order = db_models.Order(
            user_id=user.id,
            product_id=order_data.product_id,
            quantity=order_data.quantity,
            amount=order_amount,
            status="success"
        )
        db.add(db_order)
        await db.flush() # Flush to get the order id

        # Perform transaction
        purchase_transaction = db_models.Transaction(
            user_id=user.id,
            order_id=db_order.id,
            type=models.TransactionType.PURCHASE,
            amount=-order_amount,
            description=f"Purchase of {product_name} (x{order_data.quantity})",
            created_at=datetime.datetime.now(datetime.UTC)
        )
        db.add(purchase_transaction)

        sale_movement = db_models.StockMovement(
            order_id=db_order.id,
            product_id=product.id,
            type=models.StockMovementType.SALE,
            quantity=-order_data.quantity,
            description=f"Sale to user {user.id}",
            created_at=datetime.datetime.now(datetime.UTC)
        )
        db.add(sale_movement)

        # Referral logic
        if order_data.referral_bot_token:
            ref_bot_result = await db.execute(select(db_models.ReferralBot).filter(db_models.ReferralBot.bot_token == order_data.referral_bot_token))
            ref_bot = ref_bot_result.scalars().first()

            if ref_bot and ref_bot.is_active:
                seller_result = await db.execute(select(db_models.User).filter(db_models.User.id == ref_bot.seller_id))
                seller = seller_result.scalars().first()

                if seller and seller.referral_program_enabled and seller.referral_percentage > 0:
                    ref_share = order_amount * (seller.referral_percentage / 100)

                    ref_transaction = db_models.RefTransaction(
                        ref_owner_id=ref_bot.owner_id,
                        seller_id=seller.id,
                        order_id=db_order.id,
                        amount=order_amount,
                        ref_share=ref_share,
                    )
                    db.add(ref_transaction)

        await db.commit()
        await db.refresh(db_order)

        new_balance = balance - order_amount

        order_model = models.Order.model_validate(db_order)
        response_data = models.BuyResponse(
            order=order_model,
            product_name=product_name,
            product_price=product_price,
            balance=new_balance
        )
        
        return success_response(response_data)
    except Exception as e:
        logging.exception("Error processing purchase")
        return error_response("Internal server error", status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.get("", response_model=List[models.OrderResponse])
async def read_orders(db: AsyncSession = Depends(database.get_db), current_user: models.User = Depends(security.get_current_active_user)):
    try:
        result = await db.execute(
            select(db_models.Order, db_models.BotUser.telegram_id, db_models.Product.name)
            .join(db_models.BotUser, db_models.Order.user_id == db_models.BotUser.id)
            .join(db_models.Product, db_models.Order.product_id == db_models.Product.id)
            .order_by(db_models.Order.created_at.desc())
        )
        orders = result.all()
        
        response = [
            models.OrderResponse(
                id=order.id,
                user_id=order.user_id,
                product_id=order.product_id,
                quantity=order.quantity,
                amount=order.amount,
                status=order.status,
                created_at=order.created_at,
                user_telegram_id=telegram_id,
                product_name=product_name
            )
            for order, telegram_id, product_name in orders
        ]
        
        return success_response(response)
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)


@router.post("/{order_id}/cancel")
async def cancel_order(
    order_id: int,
    db: AsyncSession = Depends(database.get_db),
    _ = Depends(security.get_current_active_user)
):
    try:
        order_result = await db.execute(select(db_models.Order).filter(db_models.Order.id == order_id))
        order = order_result.scalars().first()

        if order is None:
            return error_response("Order not found", status_code=status.HTTP_404_NOT_FOUND)

        if order.status == "cancelled":
            return error_response("Order is already cancelled", status_code=status.HTTP_400_BAD_REQUEST)

        # Create a return stock movement
        return_movement = db_models.StockMovement(
            order_id=order.id,
            product_id=order.product_id,
            type=models.StockMovementType.RETURN,
            quantity=order.quantity,
            description=f"Return for cancelled order {order.id}",
            created_at=datetime.datetime.now(datetime.UTC)
        )
        db.add(return_movement)

        # Create a refund transaction
        refund_transaction = db_models.Transaction(
            user_id=order.user_id,
            order_id=order.id,
            type=models.TransactionType.DEPOSIT,
            amount=order.amount,
            description=f"Refund for cancelled order {order.id}",
            created_at=datetime.datetime.now(datetime.UTC)
        )
        db.add(refund_transaction)

        order.status = "cancelled"
        db.add(order)

        await db.commit()

        return success_response({"message": "Order cancelled successfully"})
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)