package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"
	"github.com/gin-gonic/gin"
)

func RegisterStoreBalanceRoutes(router *gin.Engine, handler *handlers.StoreBalanceHandler, authMiddleware *middleware.AuthMiddleware) {
	admin := router.Group("/api/admin")
	admin.Use(authMiddleware.RequireAuth)
	{
		admin.GET("/store-balance", middleware.PermissionMiddleware("store_balance:read"), handler.GetStoreBalance)
	}
}
