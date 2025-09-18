package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) TransactionsRouter(router *gin.Engine, transactionHandler *handlers.TransactionHandler) {
	auth := router.Group("/api/transactions")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.tokenService, r.userRepo))
	{
		auth.GET("", transactionHandler.GetAllTransactionsHandler)
	}
}
