from fastapi import APIRouter, Depends, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, func, distinct
from datetime import datetime, timedelta

from models import models
from db import database, db_models
from security import security
from core.responses import success_response, error_response

router = APIRouter()

@router.get("/stats", response_model=models.DashboardStats)
async def get_dashboard_stats(
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    try:
        # Total users
        total_users_result = await db.execute(select(func.count(db_models.BotUser.id)).filter(db_models.BotUser.is_deleted.is_(False)))
        total_users = total_users_result.scalar_one()

        # Users with purchases
        users_with_purchases_result = await db.execute(
            select(func.count(distinct(db_models.Order.user_id)))
        )
        users_with_purchases = users_with_purchases_result.scalar_one()

        # Available products
        
        # First, get all product IDs
        product_ids_result = await db.execute(select(db_models.Product.id))
        product_ids = product_ids_result.scalars().all()

        available_products = 0
        for product_id in product_ids:
            stock_result = await db.execute(
                select(func.sum(db_models.StockMovement.quantity)).filter(db_models.StockMovement.product_id == product_id)
            )
            stock = stock_result.scalar_one_or_none() or 0
            if stock > 0:
                available_products += 1


        stats = models.DashboardStats(
            total_users=total_users,
            users_with_purchases=users_with_purchases,
            available_products=available_products
        )
        return success_response(stats.model_dump())
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.get("/sales-over-time")
async def get_sales_over_time(
    start_date: datetime,
    end_date: datetime,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    try:
        # Ensure end_date is inclusive
        end_date = end_date + timedelta(days=1)

        # Products sold
        products_sold_result = await db.execute(
            select(func.count(db_models.Order.id)).filter(
                db_models.Order.created_at >= start_date,
                db_models.Order.created_at < end_date
            )
        )
        products_sold = products_sold_result.scalar_one()

        # Total revenue
        total_revenue_result = await db.execute(
            select(func.sum(db_models.Order.amount)).filter(
                db_models.Order.created_at >= start_date,
                db_models.Order.created_at < end_date
            )
        )
        total_revenue = total_revenue_result.scalar_one() or 0

        sales_data = models.SalesOverTime(
            products_sold=products_sold,
            total_revenue=total_revenue
        )
        return success_response(sales_data.model_dump())
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)
