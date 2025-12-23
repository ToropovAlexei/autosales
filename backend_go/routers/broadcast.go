package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterBroadcastRoutes(router *gin.Engine, broadcastHandler *handlers.BroadcastHandler, authMiddleware *middleware.AuthMiddleware) {
	broadcasts := router.Group("/api/admin/broadcasts")
	broadcasts.Use(authMiddleware.RequireAuth)
	{
		broadcasts.GET("/users", middleware.PermissionMiddleware("broadcasts:manage"), broadcastHandler.GetUsers)
		broadcasts.POST("/send", middleware.PermissionMiddleware("broadcasts:manage"), broadcastHandler.SendBroadcast)
	}
}
