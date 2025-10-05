package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) PaymentRouter(router *gin.Engine, handler *handlers.PaymentHandler) {
	api := router.Group("/api")

	// Public endpoints for payment processing
	api.POST("/webhooks/:gateway_name", handler.WebhookHandler)

	// Endpoints requiring service API key auth
	serviceAuth := api.Group("")
	serviceAuth.Use(middleware.ServiceTokenMiddleware(r.appSettings))
	serviceAuth.GET("/gateways", handler.GetGatewaysHandler)
	serviceAuth.POST("/deposit/invoice", handler.CreateInvoiceHandler)
	serviceAuth.PATCH("/invoices/:order_id/message-id", handler.SetInvoiceMessageIDHandler)
}
