package routers

import (
	"frbktg/backend_go/handlers"

	"github.com/gin-gonic/gin"
)

func RegisterAuthRoutes(router *gin.Engine, authHandler *handlers.AuthHandler) {
	auth := router.Group("/api/auth")
	{
		auth.POST("/login", authHandler.LoginHandler)
	}
}
