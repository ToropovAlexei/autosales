package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterTransactionRoutes(router *gin.Engine, transactionHandler *handlers.TransactionHandler, authMiddleware *middleware.AuthMiddleware) {
	transactions := router.Group("/api/transactions")
	transactions.Use(authMiddleware.RequireAuth)
	{
		transactions.GET("", transactionHandler.GetAllTransactionsHandler)
	}
}
