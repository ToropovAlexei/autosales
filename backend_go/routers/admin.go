package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterAdminRoutes(router *gin.Engine, adminHandler *handlers.AdminHandler, paymentHandler *handlers.PaymentHandler, authMiddleware *middleware.AuthMiddleware) {
	admin := router.Group("/api/admin")
	admin.Use(authMiddleware.RequireAuth)
	{
		admin.GET("/bot-users", middleware.PermissionMiddleware("users:read"), adminHandler.GetBotUsers)
		admin.GET("/bot-users/:telegram_id", middleware.PermissionMiddleware("users:read"), adminHandler.GetBotUser)
		admin.PATCH("/bot-users/:telegram_id/toggle-block", middleware.PermissionMiddleware("users:update"), adminHandler.ToggleBlockUser)

		admin.GET("/payment-providers", middleware.PermissionMiddleware("settings:read"), paymentHandler.AdminGetGatewaysHandler)
	}
}
