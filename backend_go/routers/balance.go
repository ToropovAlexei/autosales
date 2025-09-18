package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) BalanceRouter(router *gin.Engine, balanceHandler *handlers.BalanceHandler) {
	service := router.Group("/api/balance")
	service.Use(middleware.ServiceTokenMiddleware(r.appSettings))
	service.POST("/deposit", balanceHandler.DepositBalanceHandler)

	router.POST("/api/balance/webhook", balanceHandler.PaymentWebhookHandler)
}
