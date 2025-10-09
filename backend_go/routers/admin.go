package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterAdminRoutes(router *gin.Engine, adminHandler *handlers.AdminHandler, authMiddleware *middleware.AuthMiddleware) {
	admin := router.Group("/api/admin")
	admin.Use(authMiddleware.RequireAuth, middleware.AdminMiddleware())
	{
		admin.GET("/bot-users", adminHandler.GetBotUsersHandler)
	}
}
