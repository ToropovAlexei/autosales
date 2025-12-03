package routers

import (
	"frbktg/backend_go/config"
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterAuthRoutes(router *gin.Engine, authHandler *handlers.AuthHandler, authMiddleware *middleware.AuthMiddleware, appSettings *config.Config) {
	auth := router.Group("/api/auth")
	{
		auth.POST("/login", authHandler.LoginHandler)
		auth.POST("/verify-2fa", authHandler.Verify2FAHandler)
		auth.POST("/logout", authMiddleware.RequireAuth, authHandler.LogoutHandler)
	}

	botAuth := router.Group("/api/bot/auth")
	botAuth.Use(middleware.ServiceTokenMiddleware(appSettings))
	{
		botAuth.POST("/initiate", authHandler.InitiateBotAdminAuthHandler)
		botAuth.POST("/complete", authHandler.CompleteBotAdminAuthHandler)
	}
}
