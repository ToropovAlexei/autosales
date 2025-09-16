package routers

import (
	"net/http"
	"strconv"

	"frbktg/backend_go/db"
	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
)

func UsersRouter(router *gin.Engine) {
	auth := router.Group("/api/users")
	auth.Use(middleware.AuthMiddleware())
	{
		auth.GET("/me", getMeHandler)
		auth.PUT("/me/referral-settings", updateReferralSettingsHandler)
	}

	service := router.Group("/api/users")
	service.Use(middleware.ServiceTokenMiddleware())
	{
		service.POST("/register", registerBotUserHandler)
		service.GET("/:id", getBotUserHandler)
		service.GET("/:id/balance", getBalanceHandler)
		service.GET("/:id/transactions", getUserTransactionsHandler)
		service.PUT("/:id/captcha-status", updateUserCaptchaStatusHandler)
		service.GET("/seller-settings", getSellerSettingsHandler)
	}
}

func getMeHandler(c *gin.Context) {
	user, exists := c.Get("user")
	if !exists {
		errorResponse(c, http.StatusUnauthorized, "User not found in context")
		return
	}
	successResponse(c, http.StatusOK, user)
}

type ReferralSettings struct {
	ReferralProgramEnabled bool    `json:"referral_program_enabled"`
	ReferralPercentage     float64 `json:"referral_percentage"`
}

func updateReferralSettingsHandler(c *gin.Context) {
	user, exists := c.Get("user")
	if !exists {
		errorResponse(c, http.StatusUnauthorized, "User not found in context")
		return
	}

	currentUser := user.(models.User)
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

	if err := db.DB.Model(&currentUser).Updates(models.User{
		ReferralProgramEnabled: json.ReferralProgramEnabled,
		ReferralPercentage:     json.ReferralPercentage,
	}).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	successResponse(c, http.StatusOK, gin.H{"message": "Referral settings updated successfully"})
}

func registerBotUserHandler(c *gin.Context) {
	var json models.BotUser
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	var existingUser models.BotUser
	db.DB.Where("telegram_id = ?", json.TelegramID).First(&existingUser)

	if existingUser.ID != 0 {
		if !existingUser.IsDeleted {
			successResponse(c, http.StatusOK, gin.H{
				"user":               existingUser,
				"is_new":             false,
				"has_passed_captcha": existingUser.HasPassedCaptcha,
			})
			return
		} else {
			existingUser.IsDeleted = false
			existingUser.HasPassedCaptcha = false
			db.DB.Save(&existingUser)
			successResponse(c, http.StatusCreated, gin.H{
				"user":               existingUser,
				"is_new":             true,
				"has_passed_captcha": false,
			})
			return
		}
	}

	newUser := models.BotUser{TelegramID: json.TelegramID, HasPassedCaptcha: false}
	db.DB.Create(&newUser)

	successResponse(c, http.StatusCreated, gin.H{
		"user":               newUser,
		"is_new":             true,
		"has_passed_captcha": false,
	})
}

func getBotUserHandler(c *gin.Context) {
	var user models.BotUser
	if err := db.DB.Where("id = ? AND is_deleted = ?", c.Param("id"), false).First(&user).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return
	}
	successResponse(c, http.StatusOK, user)
}

func getBalanceHandler(c *gin.Context) {
	var user models.BotUser
	if err := db.DB.Where("telegram_id = ? AND is_deleted = ?", c.Param("id"), false).First(&user).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return
	}

	var balance float64
	db.DB.Model(&models.Transaction{}).Where("user_id = ?", user.ID).Select("sum(amount)").Row().Scan(&balance)

	successResponse(c, http.StatusOK, gin.H{"balance": balance})
}

func getUserTransactionsHandler(c *gin.Context) {
	var user models.BotUser
	if err := db.DB.Where("telegram_id = ? AND is_deleted = ?", c.Param("id"), false).First(&user).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return
	}

	var transactions []models.Transaction
	db.DB.Where("user_id = ?", user.ID).Order("created_at desc").Find(&transactions)

	successResponse(c, http.StatusOK, transactions)
}

func updateUserCaptchaStatusHandler(c *gin.Context) {
	var user models.BotUser
	id, _ := strconv.Atoi(c.Param("id"))
	if err := db.DB.First(&user, id).Error; err != nil {
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
	db.DB.Save(&user)

	successResponse(c, http.StatusOK, gin.H{"message": "Captcha status updated successfully"})
}

func getSellerSettingsHandler(c *gin.Context) {
	var seller models.User
	if err := db.DB.Where("role = ?", models.Admin).First(&seller).Error; err != nil {
		if err := db.DB.First(&seller).Error; err != nil {
			errorResponse(c, http.StatusNotFound, "Seller not found")
			return
		}
	}

	successResponse(c, http.StatusOK, gin.H{
		"id":                         seller.ID,
		"referral_program_enabled": seller.ReferralProgramEnabled,
		"referral_percentage":      seller.ReferralPercentage,
	})
}
