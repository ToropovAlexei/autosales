package routers

import (
	"frbktg/backend_go/config"
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterBotRoutes(router *gin.Engine, botHandler *handlers.BotHandler, productHandler *handlers.ProductHandler, orderHandler *handlers.OrderHandler, authMiddleware *middleware.AuthMiddleware, appSettings *config.Config) {
	botRoutes := router.Group("/api/bot")
	botRoutes.Use(middleware.ServiceTokenMiddleware(appSettings))
	{
		botRoutes.GET("/products", productHandler.GetProductsForBotHandler)
		botRoutes.GET("/products/:id", productHandler.GetProductForBotHandler)
		botRoutes.GET("/orders/:id", orderHandler.GetOrderHandler)
		botRoutes.POST("/invoices/:order_id/confirm", botHandler.ConfirmPayment)
		botRoutes.POST("/invoices/:order_id/cancel", botHandler.CancelPayment)
	}

	botsRoutes := router.Group("/api/bots")
	botsRoutes.Use(middleware.ServiceTokenMiddleware(appSettings))
	{
		botsRoutes.GET("", botHandler.GetAllBotsAdminHandler)
		botsRoutes.GET("/main", botHandler.GetMainBotsHandler)
	}
}
