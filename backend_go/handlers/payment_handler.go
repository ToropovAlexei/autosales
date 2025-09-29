package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"

	"github.com/gin-gonic/gin"
)

type PaymentHandler struct {
	paymentService services.PaymentService
}

func NewPaymentHandler(paymentService services.PaymentService) *PaymentHandler {
	return &PaymentHandler{paymentService: paymentService}
}

// DTO for the response of the /gateways endpoint
type gatewayDTO struct {
	Name        string `json:"name"`
	DisplayName string `json:"display_name"`
}

func (h *PaymentHandler) GetGatewaysHandler(c *gin.Context) {
	gateways := h.paymentService.GetAvailableGateways()
	var response []gatewayDTO
	for _, gw := range gateways {
		response = append(response, gatewayDTO{
			Name:        gw.GetName(),
			DisplayName: gw.GetDisplayName(),
		})
	}
	responses.SuccessResponse(c, http.StatusOK, response)
}

// DTO for the /deposit/invoice endpoint
type createInvoicePayload struct {
	GatewayName string  `json:"gateway_name" binding:"required"`
	Amount      float64 `json:"amount" binding:"required,gt=0"`
	BotUserID   uint    `json:"bot_user_id" binding:"required"`
}

func (h *PaymentHandler) CreateInvoiceHandler(c *gin.Context) {
	var json createInvoicePayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	invoice, err := h.paymentService.CreateInvoice(json.BotUserID, json.GatewayName, json.Amount)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusCreated, invoice)
}

func (h *PaymentHandler) WebhookHandler(c *gin.Context) {
	gatewayName := c.Param("gateway_name")
	if gatewayName == "" {
		c.Error(&apperrors.ErrValidation{Message: "Gateway name is missing"})
		return
	}

	if err := h.paymentService.HandleWebhook(gatewayName, c.Request); err != nil {
		c.Error(err)
		return
	}

	c.Status(http.StatusOK)
}
