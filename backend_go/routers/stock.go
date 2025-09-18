package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) StockRouter(router *gin.Engine, stockHandler *handlers.StockHandler) {
	auth := router.Group("/api/stock")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.db))
	{
		auth.GET("/movements", stockHandler.GetStockMovementsHandler)
	}
}
