
from typing import List, Optional

from fastapi import APIRouter, Depends, HTTPException

from models import models
from db import database
from security import security

router = APIRouter()

@router.get("", response_model=List[models.Product])
async def read_products(category_id: Optional[int] = None):
    products = list(database.DB["products"].values())
    if category_id:
        products = [p for p in products if p["category_id"] == category_id]
    return products

@router.post("", response_model=models.Product, status_code=201)
async def create_product(
    product: models.ProductCreate,
    current_user: models.User = Depends(security.get_current_active_user)
):
    if product.category_id not in database.DB["categories"]:
        raise HTTPException(status_code=400, detail="Category not found")
    
    new_id = database.get_next_id("products")
    new_product = models.Product(
        id=new_id, 
        name=product.name, 
        category_id=product.category_id, 
        price=product.price, 
        stock=product.stock
    ).model_dump()
    database.DB["products"][new_id] = new_product
    return new_product

@router.get("/{product_id}", response_model=models.Product)
async def read_product(product_id: int):
    product = database.DB["products"].get(product_id)
    if product is None:
        raise HTTPException(status_code=404, detail="Product not found")
    return product

@router.put("/{product_id}", response_model=models.Product)
async def update_product(
    product_id: int,
    product_update: models.ProductCreate,
    current_user: models.User = Depends(security.get_current_active_user)
):
    if product_id not in database.DB["products"]:
        raise HTTPException(status_code=404, detail="Product not found")
    if product_update.category_id not in database.DB["categories"]:
        raise HTTPException(status_code=400, detail="Category not found")

    updated_product = models.Product(id=product_id, **product_update.model_dump()).model_dump()
    database.DB["products"][product_id] = updated_product
    return updated_product

@router.delete("/{product_id}", status_code=204)
async def delete_product(
    product_id: int,
    current_user: models.User = Depends(security.get_current_active_user)
):
    if product_id not in database.DB["products"]:
        raise HTTPException(status_code=404, detail="Product not found")
    
    del database.DB["products"][product_id]
    return
