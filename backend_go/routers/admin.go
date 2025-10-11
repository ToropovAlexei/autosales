package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterAdminRoutes(router *gin.Engine, adminHandler *handlers.AdminHandler, authMiddleware *middleware.AuthMiddleware) {
	admin := router.Group("/api/admin")
	admin.Use(authMiddleware.RequireAuth)
	admin.Use(middleware.PermissionMiddleware("rbac:manage"))
	{
		admin.GET("/bot-users", adminHandler.GetBotUsers)
		admin.GET("/bot-users/:telegram_id", adminHandler.GetBotUser)
		admin.PATCH("/bot-users/:telegram_id/toggle-block", adminHandler.ToggleBlockUser)
	}
}
