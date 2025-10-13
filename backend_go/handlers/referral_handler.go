package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

type ReferralHandler struct {
	referralService services.ReferralService
}

func NewReferralHandler(referralService services.ReferralService) *ReferralHandler {
	return &ReferralHandler{referralService: referralService}
}

type referralBotCreatePayload struct {
	OwnerID  int64  `json:"owner_id" binding:"required"`
	BotToken string `json:"bot_token" binding:"required"`
}

func (h *ReferralHandler) CreateReferralBotHandler(c *gin.Context) {
	var json referralBotCreatePayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	bot, err := h.referralService.CreateReferralBot(json.OwnerID, json.BotToken)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusCreated, bot)
}

func (h *ReferralHandler) GetReferralBotsHandler(c *gin.Context) {
	bots, err := h.referralService.GetAllReferralBots()
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, bots)
}

func (h *ReferralHandler) GetAllReferralBotsAdminHandler(c *gin.Context) {
	bots, err := h.referralService.GetAllAdminInfo()
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, bots)
}

type updateBotStatusPayload struct {
	IsActive bool `json:"is_active"`
}

type botActionPayload struct {
	TelegramID int64 `json:"telegram_id" binding:"required"`
}

func (h *ReferralHandler) UpdateReferralBotStatusHandler(c *gin.Context) {
	_, exists := c.Get("user")
	if !exists {
		c.Error(apperrors.ErrForbidden)
		return
	}

	botID, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid bot ID"})
		return
	}

	var json updateBotStatusPayload
	if err := bindJSON(c, &json); err != nil {
		c.Error(err)
		return
	}

	// First, get the bot to find out the ownerID
	bot, err := h.referralService.GetReferralBotByID(uint(botID))
	if err != nil {
		c.Error(err)
		return
	}

	updatedBot, err := h.referralService.UpdateReferralBotStatus(uint(botID), bot.OwnerID, json.IsActive)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, updatedBot)
}

type updateBotPercentagePayload struct {
	Percentage float64 `json:"percentage"`
}

func (h *ReferralHandler) UpdateReferralBotPercentageHandler(c *gin.Context) {
	_, exists := c.Get("user")
	if !exists {
		c.Error(apperrors.ErrForbidden)
		return
	}

	botID, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid bot ID"})
		return
	}

	var json updateBotPercentagePayload
	if err := bindJSON(c, &json); err != nil {
		c.Error(err)
		return
	}

	bot, err := h.referralService.UpdateReferralBotPercentage(uint(botID), json.Percentage)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, bot)
}

func (h *ReferralHandler) ServiceSetPrimaryBotHandler(c *gin.Context) {
	botID, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(http.StatusBadRequest, "", err), Message: "Invalid bot ID"})
		return
	}

	var json botActionPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(http.StatusBadRequest, "", err), Message: err.Error()})
		return
	}

	if err := h.referralService.ServiceSetPrimary(uint(botID), json.TelegramID); err != nil {
		c.Error(err)
		return
	}

	// Return the updated list of bots for the user
	bots, err := h.referralService.GetReferralBotsByTelegramID(json.TelegramID)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, bots)
}

func (h *ReferralHandler) ServiceDeleteReferralBotHandler(c *gin.Context) {
	botID, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(http.StatusBadRequest, "", err), Message: "Invalid bot ID"})
		return
	}

	var json botActionPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(http.StatusBadRequest, "", err), Message: err.Error()})
		return
	}

	if err := h.referralService.ServiceDeleteReferralBot(uint(botID), json.TelegramID); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusNoContent, nil)
}

func (h *ReferralHandler) GetReferralBotsByTelegramIDHandler(c *gin.Context) {
	telegramID, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid Telegram ID"})
		return
	}

	bots, err := h.referralService.GetReferralBotsByTelegramID(telegramID)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, bots)
}

func (h *ReferralHandler) GetReferralStatsHandler(c *gin.Context) {
	telegramID, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid Telegram ID"})
		return
	}

	stats, err := h.referralService.GetReferralStats(telegramID)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, stats)
}
