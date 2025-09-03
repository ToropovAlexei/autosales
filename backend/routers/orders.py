
from fastapi import APIRouter, Depends, HTTPException

from models import models
from db import database
from security import security

router = APIRouter()

@router.post("/buy-from-balance", response_model=models.Order)
async def buy_from_balance(
    order_data: models.OrderCreate,
    _ = Depends(security.verify_service_token)
):
    user = database.DB["bot_users"].get(order_data.user_id)
    if user is None:
        raise HTTPException(status_code=404, detail="Bot user not found")

    product = database.DB["products"].get(order_data.product_id)
    if product is None:
        raise HTTPException(status_code=404, detail="Product not found")

    if product["stock"] <= 0:
        raise HTTPException(status_code=400, detail="Product out of stock")

    if user["balance"] < product["price"]:
        raise HTTPException(status_code=400, detail="Insufficient balance")

    # Perform transaction
    user["balance"] -= product["price"]
    product["stock"] -= 1

    new_id = database.get_next_id("orders")
    new_order = models.Order(
        id=new_id,
        user_id=order_data.user_id,
        product_id=order_data.product_id,
        amount=product["price"],
        status="success"
    ).model_dump()
    database.DB["orders"][new_id] = new_order

    return new_order
