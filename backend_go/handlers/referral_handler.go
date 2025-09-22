package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
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
	SellerID uint   `json:"seller_id" binding:"required"`
	BotToken string `json:"bot_token" binding:"required"`
}

func (h *ReferralHandler) CreateReferralBotHandler(c *gin.Context) {
	var json referralBotCreatePayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	bot, err := h.referralService.CreateReferralBot(json.OwnerID, json.SellerID, json.BotToken)
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

func (h *ReferralHandler) GetReferralBotsAdminHandler(c *gin.Context) {
	user, exists := c.Get("user")
	if !exists {
		c.Error(apperrors.ErrForbidden)
		return
	}
	currentUser, ok := user.(models.User)
	if !ok {
		c.Error(apperrors.ErrForbidden)
		return
	}

	if currentUser.Role != models.Admin && currentUser.Role != models.Seller {
		c.Error(apperrors.ErrForbidden)
		return
	}

	bots, err := h.referralService.GetAdminInfoForSeller(currentUser.ID)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, bots)
}

type updateBotStatusPayload struct {
	IsActive bool `json:"is_active"`
}

func (h *ReferralHandler) UpdateReferralBotStatusHandler(c *gin.Context) {
	user, exists := c.Get("user")
	if !exists {
		c.Error(apperrors.ErrForbidden)
		return
	}
	currentUser, ok := user.(models.User)
	if !ok {
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

	bot, err := h.referralService.UpdateReferralBotStatus(uint(botID), currentUser.ID, json.IsActive)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, bot)
}

func (h *ReferralHandler) SetPrimaryBotHandler(c *gin.Context) {
	user, exists := c.Get("user")
	if !exists {
		c.Error(apperrors.ErrForbidden)
		return
	}
	currentUser, ok := user.(models.User)
	if !ok {
		c.Error(apperrors.ErrForbidden)
		return
	}

	botID, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid bot ID"})
		return
	}

	if err := h.referralService.SetPrimary(uint(botID), currentUser.ID); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusNoContent, nil)
}

func (h *ReferralHandler) DeleteReferralBotHandler(c *gin.Context) {
	user, exists := c.Get("user")
	if !exists {
		c.Error(apperrors.ErrForbidden)
		return
	}
	currentUser, ok := user.(models.User)
	if !ok {
		c.Error(apperrors.ErrForbidden)
		return
	}

	botID, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid bot ID"})
		return
	}

	if err := h.referralService.DeleteReferralBot(uint(botID), currentUser.ID); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusNoContent, nil)
}