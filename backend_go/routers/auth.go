package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterAuthRoutes(router *gin.Engine, authHandler *handlers.AuthHandler, authMiddleware *middleware.AuthMiddleware) {
	auth := router.Group("/api/auth")
	{
		auth.POST("/login", authHandler.LoginHandler)
		auth.POST("/logout", authMiddleware.RequireAuth, authHandler.LogoutHandler)
	}
}
