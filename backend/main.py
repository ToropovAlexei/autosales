
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from routers import auth, categories, products, users, balance, orders, admin, transactions, stock
from config import settings

app = FastAPI(
    title="Seller Panel and TG Bot API",
    description="The API for the seller panel and Telegram bot.",
    version="0.1.0",
    docs_url="/api/docs",
    redoc_url="/api/redoc",
    openapi_url="/api/openapi.json"
)

# Configure CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.CORS_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include routers
app.include_router(auth.router, prefix="/api/auth", tags=["auth"])
app.include_router(categories.router, prefix="/api/categories", tags=["categories"])
app.include_router(products.router, prefix="/api/products", tags=["products"])
app.include_router(users.router, prefix="/api/users", tags=["users"])
app.include_router(balance.router, prefix="/api/balance", tags=["balance"])
app.include_router(orders.router, prefix="/api/orders", tags=["orders"])
app.include_router(admin.router, prefix="/api/admin", tags=["admin"])
app.include_router(transactions.router, prefix="/api/transactions", tags=["transactions"])
app.include_router(stock.router, prefix="/api/stock", tags=["stock"])

@app.get("/api")
def read_root():
    return {"message": "Welcome to the API"}
