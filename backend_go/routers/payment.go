package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterPaymentRoutes(router *gin.Engine, handler *handlers.PaymentHandler, authMiddleware *middleware.AuthMiddleware) {
	api := router.Group("/api")
	// Public endpoints for payment processing
	api.POST("/webhooks/:gateway_name", handler.WebhookHandler)

	// Endpoints requiring service API key auth
	// serviceAuth := router.Group("")
	// serviceAuth.Use(middleware.ServiceTokenMiddleware(r.appSettings))
	api.GET("/gateways", handler.GetGatewaysHandler)                                // TODO: fix this
	api.POST("/deposit/invoice", handler.CreateInvoiceHandler)                      // TODO: fix this
	api.PATCH("/invoices/:order_id/message-id", handler.SetInvoiceMessageIDHandler) // TODO: fix this
	api.GET("/invoices/:invoice_id", handler.GetInvoiceByIDHandler)                 // New route to fetch a single invoice by ID
	api.POST("/invoices/:order_id/submit-receipt", handler.SubmitReceiptLinkHandler)
}
