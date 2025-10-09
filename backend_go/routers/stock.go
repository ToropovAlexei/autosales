package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterStockRoutes(router *gin.Engine, stockHandler *handlers.StockHandler, authMiddleware *middleware.AuthMiddleware) {
	stock := router.Group("/api/stock")
	stock.Use(authMiddleware.RequireAuth)
	{
		stock.GET("/movements", stockHandler.GetStockMovementsHandler)
	}
}
