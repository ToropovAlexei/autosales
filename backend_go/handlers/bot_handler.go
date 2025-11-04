package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"

	"github.com/gin-gonic/gin"
)

type BotHandler struct {
	botService services.BotService
}

func NewBotHandler(botService services.BotService) *BotHandler {
	return &BotHandler{botService: botService}
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

func (h *BotHandler) GetMainBotsHandler(c *gin.Context) {
	bots, err := h.botService.GetMainBots()
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, bots)
}
