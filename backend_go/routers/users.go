package routers

import (
	"errors"
	"net/http"
	"strconv"

	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

func (r *Router) UsersRouter(router *gin.Engine) {
	auth := router.Group("/api/users")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.db))
	{
		auth.GET("/me", r.getMeHandler)
		auth.PUT("/me/referral-settings", r.updateReferralSettingsHandler)
	}

	service := router.Group("/api/users")
	service.Use(middleware.ServiceTokenMiddleware(r.appSettings))
	{
		service.POST("/register", r.registerBotUserHandler)
		service.GET("/:id", r.getBotUserHandler)
		service.GET("/:id/balance", r.getBalanceHandler)
		service.GET("/:id/transactions", r.getUserTransactionsHandler)
		service.PUT("/:id/captcha-status", r.updateUserCaptchaStatusHandler)
		service.GET("/seller-settings", r.getSellerSettingsHandler)
	}
}

func (r *Router) getMeHandler(c *gin.Context) {
	user, exists := c.Get("user")
	if !exists {
		errorResponse(c, http.StatusUnauthorized, "User not found in context")
		return
	}
	currentUser, ok := user.(models.User)
	if !ok {
		errorResponse(c, http.StatusInternalServerError, "Invalid user type in context")
		return
	}
	response := models.UserResponse{
		ID:                     currentUser.ID,
		Email:                  currentUser.Email,
		IsActive:               currentUser.IsActive,
		Role:                   currentUser.Role,
		ReferralProgramEnabled: currentUser.ReferralProgramEnabled,
		ReferralPercentage:     currentUser.ReferralPercentage,
	}
	successResponse(c, http.StatusOK, response)
}

type ReferralSettings struct {
	ReferralProgramEnabled bool    `json:"referral_program_enabled"`
	ReferralPercentage     float64 `json:"referral_percentage"`
}

func (r *Router) updateReferralSettingsHandler(c *gin.Context) {
	user, exists := c.Get("user")
	if !exists {
		errorResponse(c, http.StatusUnauthorized, "User not found in context")
		return
	}

	currentUser, ok := user.(models.User)
	if !ok {
		errorResponse(c, http.StatusInternalServerError, "Invalid user type in context")
		return
	}
	if currentUser.Role != models.Admin && currentUser.Role != models.Seller {
		errorResponse(c, http.StatusForbidden, "Not enough permissions")
		return
	}

	var json ReferralSettings
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	if json.ReferralPercentage < 0 || json.ReferralPercentage > 100 {
		errorResponse(c, http.StatusBadRequest, "Referral percentage must be between 0 and 100")
		return
	}

	if err := r.db.Model(&currentUser).Updates(models.User{
		ReferralProgramEnabled: json.ReferralProgramEnabled,
		ReferralPercentage:     json.ReferralPercentage,
	}).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	successResponse(c, http.StatusOK, gin.H{"message": "Referral settings updated successfully"})
}

func (r *Router) registerBotUserHandler(c *gin.Context) {
	var json models.BotUser
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	var existingUser models.BotUser
	r.db.Where("telegram_id = ?", json.TelegramID).First(&existingUser)

	if existingUser.ID != 0 {
		r.handleExistingBotUser(c, existingUser)
		return
	}

	newUser := models.BotUser{TelegramID: json.TelegramID, HasPassedCaptcha: false}
	r.db.Create(&newUser)

	response := models.BotUserResponse{
		ID:               newUser.ID,
		TelegramID:       newUser.TelegramID,
		IsDeleted:        newUser.IsDeleted,
		HasPassedCaptcha: newUser.HasPassedCaptcha,
		Balance:          0,
	}

	successResponse(c, http.StatusCreated, gin.H{
		"user":               response,
		"is_new":             true,
		"has_passed_captcha": false,
	})
}

func (r *Router) handleExistingBotUser(c *gin.Context, existingUser models.BotUser) {
	if !existingUser.IsDeleted {
		var balance float64
		if err := r.db.Model(&models.Transaction{}).Where("user_id = ?", existingUser.ID).Select("sum(amount)").
			Row().Scan(&balance); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
			errorResponse(c, http.StatusInternalServerError, err.Error())
			return
		}
		response := models.BotUserResponse{
			ID:               existingUser.ID,
			TelegramID:       existingUser.TelegramID,
			IsDeleted:        existingUser.IsDeleted,
			HasPassedCaptcha: existingUser.HasPassedCaptcha,
			Balance:          balance,
		}
		successResponse(c, http.StatusOK, gin.H{
			"user":               response,
			"is_new":             false,
			"has_passed_captcha": existingUser.HasPassedCaptcha,
		})
		return
	}
	existingUser.IsDeleted = false
	existingUser.HasPassedCaptcha = false
	r.db.Save(&existingUser)
	response := models.BotUserResponse{
		ID:               existingUser.ID,
		TelegramID:       existingUser.TelegramID,
		IsDeleted:        existingUser.IsDeleted,
		HasPassedCaptcha: existingUser.HasPassedCaptcha,
		Balance:          0,
	}
	successResponse(c, http.StatusCreated, gin.H{
		"user":               response,
		"is_new":             true,
		"has_passed_captcha": false,
	})
}

func (r *Router) getBotUserHandler(c *gin.Context) {
	var user models.BotUser
	if err := r.db.Where("id = ? AND is_deleted = ?", c.Param("id"), false).First(&user).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return
	}

	var balance float64
	if err := r.db.Model(&models.Transaction{}).Where("user_id = ?", user.ID).Select("sum(amount)").
			Row().Scan(&balance); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	response := models.BotUserResponse{
		ID:               user.ID,
		TelegramID:       user.TelegramID,
		IsDeleted:        user.IsDeleted,
		HasPassedCaptcha: user.HasPassedCaptcha,
		Balance:          balance,
	}

	successResponse(c, http.StatusOK, response)
}

func (r *Router) getBalanceHandler(c *gin.Context) {
	var user models.BotUser
	if err := r.db.Where("telegram_id = ? AND is_deleted = ?", c.Param("id"), false).First(&user).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return
	}

	var balance float64
	if err := r.db.Model(&models.Transaction{}).Where("user_id = ?", user.ID).Select("sum(amount)").
			Row().Scan(&balance); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	successResponse(c, http.StatusOK, gin.H{"balance": balance})
}

func (r *Router) getUserTransactionsHandler(c *gin.Context) {
	var user models.BotUser
	if err := r.db.Where("telegram_id = ? AND is_deleted = ?", c.Param("id"), false).First(&user).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return
	}

	var transactions []models.Transaction
	r.db.Where("user_id = ?", user.ID).Order("created_at desc").Find(&transactions)

	var response []models.TransactionResponse
	for _, t := range transactions {
		response = append(response, models.TransactionResponse(t))
	}

	successResponse(c, http.StatusOK, response)
}

func (r *Router) updateUserCaptchaStatusHandler(c *gin.Context) {
	var user models.BotUser
	id, _ := strconv.Atoi(c.Param("id"))
	if err := r.db.First(&user, id).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return
	}

	var json struct {
		HasPassedCaptcha bool `json:"has_passed_captcha"`
	}
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	user.HasPassedCaptcha = json.HasPassedCaptcha
	r.db.Save(&user)

	successResponse(c, http.StatusOK, gin.H{"message": "Captcha status updated successfully"})
}

func (r *Router) getSellerSettingsHandler(c *gin.Context) {
	var seller models.User
	if err := r.db.Where("role = ?", models.Admin).First(&seller).Error; err != nil {
		if err2 := r.db.First(&seller).Error; err2 != nil {
			errorResponse(c, http.StatusNotFound, "Seller not found")
			return
		}
	}

	successResponse(c, http.StatusOK, gin.H{
		"id":                       seller.ID,
		"referral_program_enabled": seller.ReferralProgramEnabled,
		"referral_percentage":      seller.ReferralPercentage,
	})
}
