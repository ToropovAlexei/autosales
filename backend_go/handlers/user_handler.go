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

type UserHandler struct {
	userService services.UserService
}

func NewUserHandler(userService services.UserService) *UserHandler {
	return &UserHandler{userService: userService}
}

func (h *UserHandler) GetMeHandler(c *gin.Context) {
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
	response := h.userService.GetMe(currentUser)
	responses.SuccessResponse(c, http.StatusOK, response)
}

type referralSettingsPayload struct {
	ReferralProgramEnabled bool    `json:"referral_program_enabled"`
	ReferralPercentage     float64 `json:"referral_percentage"`
}

func (h *UserHandler) UpdateReferralSettingsHandler(c *gin.Context) {
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

	var json referralSettingsPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	if json.ReferralPercentage < 0 || json.ReferralPercentage > 100 {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", nil), Message: "Referral percentage must be between 0 and 100"})
		return
	}

	if err := h.userService.UpdateReferralSettings(&currentUser, json.ReferralProgramEnabled, json.ReferralPercentage); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{"message": "Referral settings updated successfully"})
}

type registerBotUserPayload struct {
	TelegramID int64 `json:"telegram_id"`
}

func (h *UserHandler) RegisterBotUserHandler(c *gin.Context) {
	var json registerBotUserPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	user, balance, isNew, hasPassedCaptcha, err := h.userService.RegisterBotUser(json.TelegramID)
	if err != nil {
		c.Error(err)
		return
	}

	response := models.BotUserResponse{
		ID:               user.ID,
		TelegramID:       user.TelegramID,
		IsDeleted:        user.IsDeleted,
		HasPassedCaptcha: user.HasPassedCaptcha,
		Balance:          balance,
	}

	status := http.StatusOK
	if isNew {
		status = http.StatusCreated
	}

	responses.SuccessResponse(c, status, gin.H{
		"user":               response,
		"is_new":             isNew,
		"has_passed_captcha": hasPassedCaptcha,
	})
}

func (h *UserHandler) GetBotUserHandler(c *gin.Context) {
	id, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid user ID"})
		return
	}

	user, balance, err := h.userService.GetBotUserByTelegramID(id)
	if err != nil {
		c.Error(err)
		return
	}

	response := models.BotUserResponse{
		ID:               user.ID,
		TelegramID:       user.TelegramID,
		IsDeleted:        user.IsDeleted,
		HasPassedCaptcha: user.HasPassedCaptcha,
		Balance:          balance,
	}

	responses.SuccessResponse(c, http.StatusOK, response)
}

func (h *UserHandler) GetBalanceHandler(c *gin.Context) {
	id, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid user ID"})
		return
	}
	balance, err := h.userService.GetUserBalance(id)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{"balance": balance})
}

func (h *UserHandler) GetUserTransactionsHandler(c *gin.Context) {
	id, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid user ID"})
		return
	}
	transactions, err := h.userService.GetUserTransactions(id)
	if err != nil {
		c.Error(err)
		return
	}

	var response []models.TransactionResponse
	for _, t := range transactions {
		response = append(response, models.TransactionResponse{
			ID:          t.ID,
			UserID:      t.UserID,
			OrderID:     t.OrderID,
			Type:        t.Type,
			Amount:      t.Amount,
			CreatedAt:   t.CreatedAt,
			Description: t.Description,
		})
	}

	responses.SuccessResponse(c, http.StatusOK, response)
}

type updateUserCaptchaStatusPayload struct {
	HasPassedCaptcha bool `json:"has_passed_captcha"`
}

func (h *UserHandler) UpdateUserCaptchaStatusHandler(c *gin.Context) {
	id, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid user ID"})
		return
	}

	var json updateUserCaptchaStatusPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	if err := h.userService.UpdateUserCaptchaStatusByTelegramID(id, json.HasPassedCaptcha); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{"message": "Captcha status updated successfully"})
}

func (h *UserHandler) GetSellerSettingsHandler(c *gin.Context) {
	seller, err := h.userService.GetSellerSettings()
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{
		"id":                       seller.ID,
		"referral_program_enabled": seller.ReferralProgramEnabled,
		"referral_percentage":      seller.ReferralPercentage,
	})
}

func (h *UserHandler) GetUserSubscriptionsHandler(c *gin.Context) {
	id, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid telegram ID"})
		return
	}
	subscriptions, err := h.userService.GetUserSubscriptionsByTelegramID(id)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, subscriptions)
}

func (h *UserHandler) GetUserOrdersHandler(c *gin.Context) {
	id, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid telegram ID"})
		return
	}
	orders, err := h.userService.GetUserOrdersByTelegramID(id)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, orders)
}
