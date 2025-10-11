package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterSettingRoutes(router *gin.Engine, h *handlers.SettingHandler, authMiddleware *middleware.AuthMiddleware) {
	adminRoutes := router.Group("/api/admin")
	adminRoutes.Use(authMiddleware.RequireAuth)
	{
		adminRoutes.GET("/settings", middleware.PermissionMiddleware("settings:read"), h.GetSettings)
		adminRoutes.PUT("/settings", middleware.PermissionMiddleware("settings:edit"), h.UpdateSettings)
	}

	// Public route
	router.GET("/api/settings/public", h.GetPublicSettings)
}
