package handlers

import (
	"fmt"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

type UserHandler struct {
	userService    services.UserService
	roleService    services.RoleService
	paymentService services.PaymentService
}

func NewUserHandler(userService services.UserService, roleService services.RoleService, paymentService services.PaymentService) *UserHandler {
	return &UserHandler{userService: userService, roleService: roleService, paymentService: paymentService}
}

// @Summary      Get Current User
// @Description  Retrieves details for the currently authenticated admin/seller user.
// @Tags         Users
// @Produce      json
// @Success      200 {object} responses.ResponseSchema[models.UserResponse]
// @Failure      403 {object} responses.ErrorResponseSchema
// @Router       /me [get]
// @Security     ApiKeyAuth
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

// @Summary      Update Referral Settings
// @Description  Updates the referral program settings for the current admin/seller.
// @Tags         Users, Referrals
// @Accept       json
// @Produce      json
// @Param        settings body referralSettingsPayload true "Referral settings"
// @Success      200 {object} responses.ResponseSchema[responses.MessageResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      403 {object} responses.ErrorResponseSchema
// @Failure      500 {object} responses.ErrorResponseSchema
// @Router       /me/referral-settings [put]
// @Security     ApiKeyAuth
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

	var json referralSettingsPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	if json.ReferralPercentage < 0 || json.ReferralPercentage > 100 {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", nil), Message: "Referral percentage must be between 0 and 100"})
		return
	}

	if err := h.userService.UpdateReferralSettings(c, &currentUser, json.ReferralProgramEnabled, json.ReferralPercentage); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, responses.MessageResponse{Message: "Referral settings updated successfully"})
}

// @Summary      Get User Invoices
// @Description  Retrieves the payment invoice history for a bot user.
// @Tags         Users, Payments
// @Produce      json
// @Param        telegram_id path int true "User Telegram ID"
// @Param        page query int false "Page number for pagination" default(1)
// @Param        pageSize query int false "Number of items per page" default(10)
// @Success      200 {object} responses.ResponseSchema[models.PaginatedResult[models.PaymentInvoice]]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Router       /users/{telegram_id}/invoices [get]
// @Security     ServiceApiKeyAuth
func (h *UserHandler) GetUserInvoicesHandler(c *gin.Context) {
	telegramID, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid user ID"})
		return
	}

	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("pageSize", "10"))

	pageModel := models.Page{
		Page:     page,
		PageSize: pageSize,
	}

	invoices, err := h.paymentService.GetInvoicesByTelegramID(telegramID, pageModel)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, invoices)
}

type registerBotUserPayload struct {
	TelegramID int64  `json:"telegram_id"`
	BotName    string `json:"bot_name"`
}

// @Summary      Register a Bot User
// @Description  Registers a new user from a Telegram bot or reactivates a deleted one.
// @Tags         Users
// @Accept       json
// @Produce      json
// @Param        user body registerBotUserPayload true "User Telegram ID and Bot Name"
// @Success      201 {object} responses.ResponseSchema[responses.RegisterBotUserResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      500 {object} responses.ErrorResponseSchema
// @Router       /users/register [post]
// @Security     ServiceApiKeyAuth
func (h *UserHandler) RegisterBotUserHandler(c *gin.Context) {
	var json registerBotUserPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	user, balance, isNew, _, err := h.userService.RegisterBotUser(json.TelegramID, json.BotName)
	if err != nil {
		c.Error(err)
		return
	}

	userResponse := models.BotUserResponse{
		ID:                 user.ID,
		TelegramID:         user.TelegramID,
		IsBlocked:          user.IsBlocked,
		HasPassedCaptcha:   user.HasPassedCaptcha,
		Balance:            balance,
		RegisteredWithBot:  user.RegisteredWithBot,
		LastSeenWithBot:    user.LastSeenWithBot,
		LastSeenAt:         user.LastSeenAt,
		BotIsBlockedByUser: user.BotIsBlockedByUser,
	}

	status := http.StatusOK
	if isNew {
		status = http.StatusCreated
	}

	responses.SuccessResponse(c, status, userResponse)
}

// @Summary      Get User Balance
// @Description  Retrieves the current balance for a bot user.
// @Tags         Users, Balance
// @Produce      json
// @Param        telegram_id path int true "User Telegram ID"
// @Success      200 {object} responses.ResponseSchema[responses.BalanceResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Router       /users/{telegram_id}/balance [get]
// @Security     ServiceApiKeyAuth
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

	responses.SuccessResponse(c, http.StatusOK, responses.BalanceResponse{Balance: balance})
}

// @Summary      Get User Transactions
// @Description  Retrieves the transaction history for a bot user.
// @Tags         Users, Transactions
// @Produce      json
// @Param        telegram_id path int true "User Telegram ID"
// @Success      200 {object} responses.ResponseSchema[[]models.TransactionResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Router       /users/{telegram_id}/transactions [get]
// @Security     ServiceApiKeyAuth
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

// @Summary      Update User Captcha Status
// @Description  Updates the captcha verification status for a bot user.
// @Tags         Users
// @Accept       json
// @Produce      json
// @Param        telegram_id path int true "User Telegram ID"
// @Param        status body updateUserCaptchaStatusPayload true "Captcha status"
// @Success      200 {object} responses.ResponseSchema[responses.MessageResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Router       /users/{telegram_id}/captcha-status [put]
// @Security     ServiceApiKeyAuth
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

	responses.SuccessResponse(c, http.StatusOK, responses.MessageResponse{Message: "Captcha status updated successfully"})
}

// @Summary      Update Bot User Status
// @Description  Partially updates a bot user's status fields, such as block status.
// @Tags         Users
// @Accept       json
// @Produce      json
// @Param        telegram_id path int true "User Telegram ID"
// @Param        status body models.UpdateBotUserStatusPayload true "Status fields to update"
// @Success      200 {object} responses.ResponseSchema[responses.MessageResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Router       /users/{telegram_id}/status [patch]
// @Security     ServiceApiKeyAuth
func (h *UserHandler) UpdateBotUserStatusHandler(c *gin.Context) {
	id, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid user ID"})
		return
	}

	var payload models.UpdateBotUserStatusPayload
	if err := c.ShouldBindJSON(&payload); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	if err := h.userService.UpdateBotUserStatus(id, payload); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, responses.MessageResponse{Message: "User status updated successfully"})
}

// @Summary      Get User Subscriptions
// @Description  Retrieves a list of a bot user's active and expired subscriptions.
// @Tags         Users, Subscriptions
// @Produce      json
// @Param        telegram_id path int true "User Telegram ID"
// @Success      200 {object} responses.ResponseSchema[[]models.UserSubscription]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Router       /users/{telegram_id}/subscriptions [get]
// @Security     ServiceApiKeyAuth
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

type getBotUserPayload struct {
	BotName string `json:"bot_name"`
}

// @Summary      Get User Orders
// @Description  Retrieves the order history for a bot user.
// @Tags         Users, Orders
// @Produce      json
// @Param        telegram_id path int true "User Telegram ID"
// @Success      200 {object} responses.ResponseSchema[[]models.Order]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Router       /users/{telegram_id}/orders [get]
// @Security     ServiceApiKeyAuth
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

	// Map to DTO
	response := make([]models.UserOrderResponse, 0, len(orders))
	for _, order := range orders {
		productImageURL := ""
		if order.Product.ImageID != nil {
			productImageURL = fmt.Sprintf("/images/%s", order.Product.ImageID.String())
		}
		fulfilledImageURL := ""
		if order.FulfilledImageID != nil {
			fulfilledImageURL = fmt.Sprintf("/images/%s", order.FulfilledImageID.String())
		}
		response = append(response, models.UserOrderResponse{
			ID:                order.ID,
			ProductName:       order.Product.Name,
			Amount:            order.Amount,
			CreatedAt:         order.CreatedAt,
			FulfilledContent:  order.FulfilledContent,
			ProductImageURL:   productImageURL,
			FulfilledImageURL: fulfilledImageURL,
		})
	}

	responses.SuccessResponse(c, http.StatusOK, response)
}

func (h *UserHandler) GetMyPermissionsHandler(c *gin.Context) {
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

	permissions, err := h.roleService.GetUserFinalPermissions(currentUser.ID)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, permissions)
}

func (h *UserHandler) GetBotUserHandler(c *gin.Context) {
	id, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid user ID"})
		return
	}

	botName := c.Query("bot_name")
	if botName == "" {
		c.Error(&apperrors.ErrValidation{
			Base:    apperrors.New(400, "", nil),
			Message: "Missing bot_name query parameter",
		})
		return
	}

	user, balance, err := h.userService.GetBotUserByTelegramID(id, botName)
	if err != nil {
		c.Error(err)
		return
	}

	response := models.BotUserResponse{
		ID:                 user.ID,
		TelegramID:         user.TelegramID,
		IsBlocked:          user.IsBlocked,
		HasPassedCaptcha:   user.HasPassedCaptcha,
		Balance:            balance,
		RegisteredWithBot:  user.RegisteredWithBot,
		LastSeenWithBot:    user.LastSeenWithBot,
		LastSeenAt:         user.LastSeenAt,
		BotIsBlockedByUser: user.BotIsBlockedByUser,
	}

	responses.SuccessResponse(c, http.StatusOK, response)
}

// @Summary      Toggle Block Status of a Bot User
// @Description  Blocks or unblocks a bot user.
// @Tags         Users
// @Produce      json
// @Param        telegram_id path int true "User Telegram ID"
// @Success      200 {object} responses.ResponseSchema[responses.MessageResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Router       /users/{telegram_id}/toggle-block [patch]
// @Security     ApiKeyAuth
func (h *UserHandler) ToggleBlockUserHandler(c *gin.Context) {
	id, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid user ID"})
		return
	}

	if err := h.userService.ToggleBlockUser(c, id); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, responses.MessageResponse{Message: "User block status updated successfully"})
}
