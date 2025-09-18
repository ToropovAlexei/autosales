package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) OrdersRouter(router *gin.Engine, orderHandler *handlers.OrderHandler) {
	service := router.Group("/api/orders")
	service.Use(middleware.ServiceTokenMiddleware(r.appSettings))
	{
		service.POST("/buy-from-balance", orderHandler.BuyFromBalanceHandler)
	}

	auth := router.Group("/api/orders")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.db))
	{
		auth.GET("", orderHandler.GetOrdersHandler)
		auth.POST("/:id/cancel", orderHandler.CancelOrderHandler)
	}
}
