package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterBalanceRoutes(router *gin.Engine, balanceHandler *handlers.BalanceHandler, authMiddleware *middleware.AuthMiddleware) {
	balance := router.Group("/api/balance")
	// service.Use(middleware.ServiceTokenMiddleware(r.appSettings)) // TODO: fix this
	balance.POST("/deposit", balanceHandler.DepositBalanceHandler)

	balance.POST("/webhook", balanceHandler.PaymentWebhookHandler)
}
