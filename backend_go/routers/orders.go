package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterOrderRoutes(router *gin.Engine, orderHandler *handlers.OrderHandler, authMiddleware *middleware.AuthMiddleware) {
	orders := router.Group("/api/orders")

	// service.Use(middleware.ServiceTokenMiddleware(r.appSettings))
	orders.POST("/buy-from-balance", orderHandler.BuyFromBalanceHandler) // TODO: fix this

	orders.Use(authMiddleware.RequireAuth)
	{
		orders.GET("", middleware.PermissionMiddleware("orders:read"), orderHandler.GetOrdersHandler)
		orders.GET("/:id", middleware.PermissionMiddleware("orders:read"), orderHandler.GetOrderHandler)
		orders.POST("/:id/cancel", middleware.PermissionMiddleware("orders:update"), orderHandler.CancelOrderHandler)
	}
}
