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

type gatewayDTO struct {
	Name        string `json:"name"`
	DisplayName string `json:"display_name"`
}

// @Summary      Get Payment Gateways
// @Description  Retrieves a list of available payment gateways.
// @Tags         Payments
// @Produce      json
// @Success      200  {object}  responses.ResponseSchema[[]gatewayDTO]
// @Failure      500  {object}  responses.ErrorResponseSchema
// @Router       /gateways [get]
// @Security     ServiceApiKeyAuth
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

type createInvoicePayload struct {
	GatewayName string  `json:"gateway_name" binding:"required"`
	Amount      float64 `json:"amount" binding:"required,gt=0"`
	BotUserID   uint    `json:"bot_user_id" binding:"required"`
}

// @Summary      Create Payment Invoice
// @Description  Creates a new payment invoice for a selected gateway and amount.
// @Tags         Payments
// @Accept       json
// @Produce      json
// @Param        invoice body createInvoicePayload true "Invoice creation data"
// @Success      201  {object}  responses.ResponseSchema[services.CreateInvoiceResponse]
// @Failure      400  {object}  responses.ErrorResponseSchema
// @Failure      500  {object}  responses.ErrorResponseSchema
// @Router       /deposit/invoice [post]
// @Security     ServiceApiKeyAuth
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

type setMessageIDPayload struct {
	MessageID int64 `json:"message_id" binding:"required"`
}

// @Summary      Set Invoice Message ID
// @Description  Sets the Telegram message ID for a given payment invoice.
// @Tags         Payments
// @Accept       json
// @Param        order_id path string true "Internal Order ID of the invoice"
// @Param        payload body setMessageIDPayload true "Message ID payload"
// @Success      200
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Router       /invoices/{order_id}/message-id [patch]
// @Security     ServiceApiKeyAuth
func (h *PaymentHandler) SetInvoiceMessageIDHandler(c *gin.Context) {
	orderID := c.Param("order_id")
	var json setMessageIDPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	if err := h.paymentService.SetInvoiceMessageID(orderID, json.MessageID); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, responses.MessageResponse{Message: "Message ID updated successfully"})
}

// @Summary      Handle Gateway Webhook
// @Description  Handles incoming webhook notifications from a payment gateway.
// @Tags         Payments
// @Accept       json
// @Produce      json
// @Param        gateway_name path string true "Gateway Name"
// @Success      200
// @Failure      400  {object}  responses.ErrorResponseSchema
// @Failure      404  {object}  responses.ErrorResponseSchema
// @Failure      500  {object}  responses.ErrorResponseSchema
// @Router       /webhooks/{gateway_name} [post]
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
