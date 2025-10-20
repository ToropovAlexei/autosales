package routers

import (
	"frbktg/backend_go/config"
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterBotRoutes(router *gin.Engine, productHandler *handlers.ProductHandler, appSettings config.Settings) {
	bot := router.Group("/api/bot")
	bot.Use(middleware.ServiceTokenMiddleware(appSettings)) // Use service-level authentication
	{
		bot.GET("/products", productHandler.GetProductsForBotHandler)
	}
}
