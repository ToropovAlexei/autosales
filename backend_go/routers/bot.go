package routers

import (
	"frbktg/backend_go/config"
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterBotRoutes(router *gin.Engine, botHandler *handlers.BotHandler, authMiddleware *middleware.AuthMiddleware, appSettings *config.Config) {
	serviceBots := router.Group("/api/bots")
	serviceBots.Use(middleware.ServiceTokenMiddleware(appSettings))
	{
		serviceBots.POST("/referral", botHandler.CreateReferralBotHandler)
		serviceBots.GET("", botHandler.GetAllBotsAdminHandler)
		serviceBots.GET("/main", botHandler.GetMainBotsHandler)
	}

	adminBots := router.Group("/api/bots")
	adminBots.Use(authMiddleware.RequireAuth)
	{
		// adminBots.GET("", middleware.PermissionMiddleware("bots:read"), botHandler.GetAllBotsAdminHandler)
		// adminBots.PUT("/:id/status", middleware.PermissionMiddleware("bots:update"), botHandler.UpdateBotStatusHandler)
		// adminBots.PUT("/:id/percentage", middleware.PermissionMiddleware("bots:update"), botHandler.UpdateBotPercentageHandler)
	}
}