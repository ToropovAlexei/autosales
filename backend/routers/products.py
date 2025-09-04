from typing import List, Optional

from fastapi import APIRouter, Depends, status
from fastapi.responses import JSONResponse
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from models import models
from db import database, db_models
from security import security
from core.responses import success_response, error_response

router = APIRouter()

@router.get("")
async def read_products(category_id: Optional[int] = None, db: AsyncSession = Depends(database.get_db)):
    try:
        query = select(db_models.Product)
        if category_id:
            query = query.filter(db_models.Product.category_id == category_id)
        result = await db.execute(query)
        products = result.scalars().all()
        return success_response([models.Product.model_validate(p).model_dump() for p in products])
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.post("", status_code=status.HTTP_201_CREATED)
async def create_product(
    product: models.ProductCreate,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    try:
        # Verify category exists
        cat_result = await db.execute(select(db_models.Category).filter(db_models.Category.id == product.category_id))
        if cat_result.scalars().first() is None:
            return error_response("Category not found", status_code=status.HTTP_400_BAD_REQUEST)

        db_product = db_models.Product(**product.model_dump())
        db.add(db_product)
        await db.commit()
        await db.refresh(db_product)
        return success_response(models.Product.model_validate(db_product).model_dump(), status_code=status.HTTP_201_CREATED)
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.get("/{product_id}")
async def read_product(product_id: int, db: AsyncSession = Depends(database.get_db)):
    try:
        result = await db.execute(select(db_models.Product).filter(db_models.Product.id == product_id))
        product = result.scalars().first()
        if product is None:
            return error_response("Product not found", status_code=status.HTTP_404_NOT_FOUND)
        return success_response(models.Product.model_validate(product).model_dump())
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.put("/{product_id}")
async def update_product(
    product_id: int,
    product_update: models.ProductCreate,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    try:
        result = await db.execute(select(db_models.Product).filter(db_models.Product.id == product_id))
        db_product = result.scalars().first()
        if db_product is None:
            return error_response("Product not found", status_code=status.HTTP_404_NOT_FOUND)

        # Verify category exists
        cat_result = await db.execute(select(db_models.Category).filter(db_models.Category.id == product_update.category_id))
        if cat_result.scalars().first() is None:
            return error_response("Category not found", status_code=status.HTTP_400_BAD_REQUEST)

        for key, value in product_update.model_dump().items():
            setattr(db_product, key, value)
        
        await db.commit()
        await db.refresh(db_product)
        return success_response(models.Product.model_validate(db_product).model_dump())
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)

@router.delete("/{product_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_product(
    product_id: int,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    try:
        result = await db.execute(select(db_models.Product).filter(db_models.Product.id == product_id))
        db_product = result.scalars().first()
        if db_product is None:
            return error_response("Product not found", status_code=status.HTTP_404_NOT_FOUND)

        await db.delete(db_product)
        await db.commit()
        return JSONResponse(status_code=status.HTTP_204_NO_CONTENT)
    except Exception as e:
        return error_response(str(e), status_code=status.HTTP_500_INTERNAL_SERVER_ERROR)