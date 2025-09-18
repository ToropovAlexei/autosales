package handlers

import (
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
		responses.ErrorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	bot, err := h.referralService.CreateReferralBot(json.OwnerID, json.SellerID, json.BotToken)
	if err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	responses.SuccessResponse(c, http.StatusCreated, bot)
}

func (h *ReferralHandler) GetReferralBotsHandler(c *gin.Context) {
	bots, err := h.referralService.GetAllReferralBots()
	if err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}
	responses.SuccessResponse(c, http.StatusOK, bots)
}

func (h *ReferralHandler) GetReferralBotsAdminHandler(c *gin.Context) {
	user, exists := c.Get("user")
	if !exists {
		responses.ErrorResponse(c, http.StatusUnauthorized, "User not found in context")
		return
	}
	currentUser, ok := user.(models.User)
	if !ok {
		responses.ErrorResponse(c, http.StatusInternalServerError, "Invalid user type in context")
		return
	}

	if currentUser.Role != models.Admin && currentUser.Role != models.Seller {
		responses.ErrorResponse(c, http.StatusForbidden, "Not enough permissions")
		return
	}

	bots, err := h.referralService.GetAdminInfoForSeller(currentUser.ID)
	if err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	responses.SuccessResponse(c, http.StatusOK, bots)
}

func (h *ReferralHandler) ToggleReferralBotStatusHandler(c *gin.Context) {
	user, exists := c.Get("user")
	if !exists {
		responses.ErrorResponse(c, http.StatusUnauthorized, "User not found in context")
		return
	}
	currentUser, ok := user.(models.User)
	if !ok {
		responses.ErrorResponse(c, http.StatusInternalServerError, "Invalid user type in context")
		return
	}

	botID, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, "Invalid bot ID")
		return
	}

	bot, err := h.referralService.ToggleReferralBotStatus(uint(botID), currentUser.ID)
	if err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	responses.SuccessResponse(c, http.StatusOK, bot)
}
