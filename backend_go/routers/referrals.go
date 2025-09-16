package routers

import (
	"net/http"

	"frbktg/backend_go/db"
	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
)

func ReferralsRouter(router *gin.Engine) {
	service := router.Group("/api/referrals")
	service.Use(middleware.ServiceTokenMiddleware())
	{
		service.POST("", createReferralBotHandler)
		service.GET("", getReferralBotsHandler)
	}

	auth := router.Group("/api/referrals")
	auth.Use(middleware.AuthMiddleware())
	{
		auth.GET("/admin-list", getReferralBotsAdminHandler)
		auth.PUT("/:id", toggleReferralBotStatusHandler)
	}
}

type ReferralBotCreate struct {
	OwnerID  uint   `json:"owner_id"`
	SellerID uint   `json:"seller_id"`
	BotToken string `json:"bot_token"`
}

func createReferralBotHandler(c *gin.Context) {
	var json ReferralBotCreate
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	var owner models.BotUser
	if err := db.DB.Where("telegram_id = ?", json.OwnerID).First(&owner).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Referral owner (user) not found.")
		return
	}

	var seller models.User
	if err := db.DB.First(&seller, json.SellerID).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Seller not found.")
		return
	}

	var existingBot models.ReferralBot
	if err := db.DB.Where("bot_token = ?", json.BotToken).First(&existingBot).Error; err == nil {
		errorResponse(c, http.StatusBadRequest, "Bot with this token already exists.")
		return
	}

	dbBot := models.ReferralBot{
		OwnerID:  owner.ID,
		SellerID: json.SellerID,
		BotToken: json.BotToken,
	}

	if err := db.DB.Create(&dbBot).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	successResponse(c, http.StatusOK, dbBot)
}

func getReferralBotsHandler(c *gin.Context) {
	var bots []models.ReferralBot
	if err := db.DB.Find(&bots).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}
	successResponse(c, http.StatusOK, bots)
}

type ReferralBotAdminInfo struct {
	models.ReferralBot
	OwnerTelegramID int64   `json:"owner_telegram_id"`
	Turnover        float64 `json:"turnover"`
	Accruals        float64 `json:"accruals"`
}

func getReferralBotsAdminHandler(c *gin.Context) {
	user, _ := c.Get("user")
	currentUser := user.(models.User)

	if currentUser.Role != models.Admin && currentUser.Role != models.Seller {
		errorResponse(c, http.StatusForbidden, "Not enough permissions")
		return
	}

	var bots []ReferralBotAdminInfo

	db.DB.Table("referral_bots").
		Select("referral_bots.*, bot_users.telegram_id as owner_telegram_id, COALESCE(SUM(ref_transactions.amount), 0) as turnover, COALESCE(SUM(ref_transactions.ref_share), 0) as accruals").
		Joins("join bot_users on referral_bots.owner_id = bot_users.id").
		Joins("left join ref_transactions on referral_bots.owner_id = ref_transactions.ref_owner_id").
		Where("referral_bots.seller_id = ?", currentUser.ID).
		Group("referral_bots.id, bot_users.telegram_id").
		Scan(&bots)

	successResponse(c, http.StatusOK, bots)
}

func toggleReferralBotStatusHandler(c *gin.Context) {
	user, _ := c.Get("user")
	currentUser := user.(models.User)

	if currentUser.Role != models.Admin && currentUser.Role != models.Seller {
		errorResponse(c, http.StatusForbidden, "Not enough permissions")
		return
	}

	var bot models.ReferralBot
	if err := db.DB.First(&bot, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Referral bot not found")
		return
	}

	if bot.SellerID != currentUser.ID {
		errorResponse(c, http.StatusForbidden, "You are not the owner of this referral bot")
		return
	}

	bot.IsActive = !bot.IsActive
	db.DB.Save(&bot)

	successResponse(c, http.StatusOK, bot)
}
