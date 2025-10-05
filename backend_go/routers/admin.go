package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) AdminRouter(router *gin.Engine, adminHandler *handlers.AdminHandler) {
	admin := router.Group("/api/admin")
	admin.Use(middleware.AuthMiddleware(r.appSettings, r.tokenService, r.userRepo), middleware.AdminMiddleware())
	admin.GET("/bot-users", adminHandler.GetBotUsersHandler)
}
