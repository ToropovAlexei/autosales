package handlers

import (
	"encoding/json"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

type OrderHandler struct {
	orderService services.OrderService
	botService   services.BotService
}

func NewOrderHandler(orderService services.OrderService, botService services.BotService) *OrderHandler {
	return &OrderHandler{orderService: orderService, botService: botService}
}

// @Summary      List Orders
// @Description  Retrieves a list of all orders.
// @Tags         Orders
// @Produce      json
// @Success      200 {object} responses.ResponseSchema[[]models.OrderResponse]
// @Failure      500 {object} responses.ErrorResponseSchema
// @Router       /orders [get]
// @Security     ApiKeyAuth
func (h *OrderHandler) GetOrdersHandler(c *gin.Context) {
	var page models.Page
	if err := c.ShouldBindQuery(&page); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	var filters []models.Filter
	if filtersJSON := c.Query("filters"); filtersJSON != "" {
		if err := json.Unmarshal([]byte(filtersJSON), &filters); err != nil {
			c.Error(&apperrors.ErrValidation{Message: "Invalid filters format"})
			return
		}
	}

	orders, err := h.orderService.GetOrders(page, filters)
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, orders)
}

type BuyPayload struct {
	UserID    int64 `json:"user_id" binding:"required"`
	ProductID uint  `json:"product_id" binding:"required"`
	Quantity  int   `json:"quantity" binding:"required,gt=0"`
}

// @Summary      Buy a Product
// @Description  Creates an order and processes the purchase of a product from the user's balance.
// @Tags         Orders
// @Accept       json
// @Produce      json
// @Param        bot_name query string false "Name of the referral bot"
// @Param        purchase body BuyPayload true "Purchase data"
// @Success      200 {object} responses.ResponseSchema[services.BuyResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      402 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Failure      500 {object} responses.ErrorResponseSchema
// @Router       /orders/buy-from-balance [post]
// @Security     ServiceApiKeyAuth
func (h *OrderHandler) BuyFromBalanceHandler(c *gin.Context) {
	var json BuyPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	botName := c.Query("bot_name")
	var botID uint
	if botName != "" {
		bot, err := h.botService.FindBotByName(botName)
		if err == nil && bot != nil {
			botID = bot.ID
		}
	}

	buyRequest := services.BuyRequest{
		UserID:    json.UserID,
		ProductID: json.ProductID,
		Quantity:  json.Quantity,
		BotID:     botID,
	}

	buyResponse, err := h.orderService.BuyFromBalance(buyRequest)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, buyResponse)
}

// @Summary      Cancel an Order
// @Description  Cancels an order and refunds the user.
// @Tags         Orders
// @Produce      json
// @Param        id path int true "Order ID"
// @Success      200 {object} responses.ResponseSchema[responses.MessageResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Failure      500 {object} responses.ErrorResponseSchema
// @Router       /orders/{id}/cancel [post]
// @Security     ApiKeyAuth
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

	responses.SuccessResponse(c, http.StatusOK, responses.MessageResponse{Message: "Order cancelled successfully"})
}
