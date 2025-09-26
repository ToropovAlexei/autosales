package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

type OrderHandler struct {
	orderService services.OrderService
}

func NewOrderHandler(orderService services.OrderService) *OrderHandler {
	return &OrderHandler{orderService: orderService}
}

func (h *OrderHandler) GetOrdersHandler(c *gin.Context) {
	orders, err := h.orderService.GetOrders()
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, orders)
}

type BuyPayload struct {
	UserID           int64   `json:"user_id" binding:"required"`
	ProductID        *uint   `json:"product_id"`
	Provider         *string `json:"provider"`
	ExternalProductID *string `json:"external_product_id"`
	Quantity         int     `json:"quantity" binding:"required,gt=0"`
	ReferralBotToken *string `json:"referral_bot_token"`
}

func (h *OrderHandler) BuyFromBalanceHandler(c *gin.Context) {
	var json BuyPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	// Basic validation
	if json.ProductID == nil && (json.Provider == nil || json.ExternalProductID == nil) {
		c.Error(&apperrors.ErrValidation{Message: "either product_id or both provider and external_product_id are required"})
		return
	}

	buyRequest := services.BuyRequest{
		UserID:           json.UserID,
		ProductID:        json.ProductID,
		Provider:         json.Provider,
		ExternalProductID: json.ExternalProductID,
		Quantity:         json.Quantity,
		ReferralBotToken: json.ReferralBotToken,
	}

	buyResponse, err := h.orderService.BuyFromBalance(buyRequest)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, buyResponse)
}

func (h *OrderHandler) CancelOrderHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid order ID"})
		return
	}

	if err := h.orderService.CancelOrder(uint(id)); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{"message": "Order cancelled successfully"})
}