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

type buyFromBalancePayload struct {
	UserID           int64   `json:"user_id" binding:"required"`
	ProductID        uint    `json:"product_id" binding:"required"`
	Quantity         int     `json:"quantity" binding:"required,gt=0"`
	ReferralBotToken *string `json:"referral_bot_token"`
}

func (h *OrderHandler) BuyFromBalanceHandler(c *gin.Context) {
	var json buyFromBalancePayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	buyResponse, err := h.orderService.BuyFromBalance(json.UserID, json.ProductID, json.Quantity, json.ReferralBotToken)
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