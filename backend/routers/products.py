

from typing import List, Optional

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from models import models
from db import database, db_models
from security import security

router = APIRouter()

@router.get("", response_model=List[models.Product])
async def read_products(category_id: Optional[int] = None, db: AsyncSession = Depends(database.get_db)):
    query = select(db_models.Product)
    if category_id:
        query = query.filter(db_models.Product.category_id == category_id)
    result = await db.execute(query)
    products = result.scalars().all()
    return products

@router.post("", response_model=models.Product, status_code=201)
async def create_product(
    product: models.ProductCreate,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    # Verify category exists
    cat_result = await db.execute(select(db_models.Category).filter(db_models.Category.id == product.category_id))
    if cat_result.scalars().first() is None:
        raise HTTPException(status_code=400, detail="Category not found")

    db_product = db_models.Product(**product.model_dump())
    db.add(db_product)
    await db.commit()
    await db.refresh(db_product)
    return db_product

@router.get("/{product_id}", response_model=models.Product)
async def read_product(product_id: int, db: AsyncSession = Depends(database.get_db)):
    result = await db.execute(select(db_models.Product).filter(db_models.Product.id == product_id))
    product = result.scalars().first()
    if product is None:
        raise HTTPException(status_code=404, detail="Product not found")
    return product

@router.put("/{product_id}", response_model=models.Product)
async def update_product(
    product_id: int,
    product_update: models.ProductCreate,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    result = await db.execute(select(db_models.Product).filter(db_models.Product.id == product_id))
    db_product = result.scalars().first()
    if db_product is None:
        raise HTTPException(status_code=404, detail="Product not found")

    # Verify category exists
    cat_result = await db.execute(select(db_models.Category).filter(db_models.Category.id == product_update.category_id))
    if cat_result.scalars().first() is None:
        raise HTTPException(status_code=400, detail="Category not found")

    for key, value in product_update.model_dump().items():
        setattr(db_product, key, value)
    
    await db.commit()
    await db.refresh(db_product)
    return db_product

@router.delete("/{product_id}", status_code=204)
async def delete_product(
    product_id: int,
    db: AsyncSession = Depends(database.get_db),
    current_user: models.User = Depends(security.get_current_active_user)
):
    result = await db.execute(select(db_models.Product).filter(db_models.Product.id == product_id))
    db_product = result.scalars().first()
    if db_product is None:
        raise HTTPException(status_code=404, detail="Product not found")

    await db.delete(db_product)
    await db.commit()
    return

