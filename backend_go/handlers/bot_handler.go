package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

type BotHandler struct {
	botService     services.BotService
	paymentService services.PaymentService
}

func NewBotHandler(botService services.BotService, paymentService services.PaymentService) *BotHandler {
	return &BotHandler{botService: botService, paymentService: paymentService}
}

type referralBotCreatePayload struct {
	OwnerID  int64  `json:"owner_id" binding:"required"`
	BotToken string `json:"bot_token" binding:"required"`
}

func (h *BotHandler) CreateReferralBotHandler(c *gin.Context) {
	var json referralBotCreatePayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	bot, err := h.botService.CreateReferralBot(json.OwnerID, json.BotToken)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusCreated, bot)
}

func (h *BotHandler) GetAllBotsAdminHandler(c *gin.Context) {
	botType := c.Query("type")
	bots, err := h.botService.GetAllBots(botType)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, bots)
}

func (h *BotHandler) GetReferralBotsAdminHandler(c *gin.Context) {
	bots, err := h.botService.GetAllBots("")
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, bots)
}

func (h *BotHandler) GetMainBotsHandler(c *gin.Context) {
	bots, err := h.botService.GetMainBots()
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, bots)
}

func (h *BotHandler) DeleteBotHandler(c *gin.Context) {
	botID, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid bot ID"})
		return
	}

	if err := h.botService.DeleteBot(uint(botID)); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusNoContent, nil)
}

func (h *BotHandler) UpdateBotStatusHandler(c *gin.Context) {
	botID, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid bot ID"})
		return
	}

	var payload struct {
		IsActive bool `json:"is_active"`
	}
	if err := c.ShouldBindJSON(&payload); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	if err := h.botService.UpdateBotStatus(uint(botID), payload.IsActive); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, nil)
}

func (h *BotHandler) SetPrimaryBotHandler(c *gin.Context) {
	botID, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid bot ID"})
		return
	}

	if err := h.botService.SetPrimaryBot(uint(botID)); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, nil)
}

func (h *BotHandler) UpdateBotReferralPercentageHandler(c *gin.Context) {
	botID, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid bot ID"})
		return
	}

	var payload struct {
		Percentage float64 `json:"percentage"`
	}
	if err := c.ShouldBindJSON(&payload); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	if err := h.botService.UpdateBotReferralPercentage(uint(botID), payload.Percentage); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, nil)
}

func (h *BotHandler) ConfirmPayment(c *gin.Context) {
	orderID := c.Param("order_id")
	if orderID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "order_id is required"})
		return
	}

	if err := h.paymentService.ConfirmExternalPayment(orderID); err != nil {
		c.Error(err)
		return
	}

	c.JSON(http.StatusOK, gin.H{"success": true, "message": "payment confirmed"})
}

func (h *BotHandler) CancelPayment(c *gin.Context) {
	orderID := c.Param("order_id")
	if orderID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "order_id is required"})
		return
	}

	if err := h.paymentService.CancelExternalPayment(orderID); err != nil {
		c.Error(err)
		return
	}

	c.JSON(http.StatusOK, gin.H{"success": true, "message": "payment cancelled"})
}
