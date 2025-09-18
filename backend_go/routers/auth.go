package routers

import (
	"frbktg/backend_go/handlers"

	"github.com/gin-gonic/gin"
)

func (r *Router) AuthRouter(router *gin.Engine, authHandler *handlers.AuthHandler) {
	router.POST("/api/auth/login", authHandler.LoginHandler)
}
