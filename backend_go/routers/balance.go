package routers

import (
	"net/http"
	"time"

	"frbktg/backend_go/db"
	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
)

func BalanceRouter(router *gin.Engine) {
	service := router.Group("/api/balance")
	service.Use(middleware.ServiceTokenMiddleware())
	{
		service.POST("/deposit", depositBalanceHandler)
	}

	router.POST("/api/balance/webhook", paymentWebhookHandler)
}

type DepositRequest struct {
	UserID int64   `json:"user_id"`
	Amount float64 `json:"amount"`
}

func depositBalanceHandler(c *gin.Context) {
	var json DepositRequest
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	var user models.BotUser
	if err := db.DB.Where("telegram_id = ? AND is_deleted = ?", json.UserID, false).First(&user).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return
	}

	transaction := models.Transaction{
		UserID:      user.ID,
		Type:        models.Deposit,
		Amount:      json.Amount,
		Description: "Test deposit",
		CreatedAt:   time.Now().UTC(),
	}

	if err := db.DB.Create(&transaction).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	successResponse(c, http.StatusOK, gin.H{"message": "Balance updated successfully"})
}

type WebhookPayload struct {
	UserID int64   `json:"user_id"`
	Amount float64 `json:"amount"`
}

func paymentWebhookHandler(c *gin.Context) {
	var json WebhookPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	var user models.BotUser
	if err := db.DB.Where("telegram_id = ? AND is_deleted = ?", json.UserID, false).First(&user).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return
	}

	transaction := models.Transaction{
		UserID:      user.ID,
		Type:        models.Deposit,
		Amount:      json.Amount,
		Description: "Deposit via webhook",
		CreatedAt:   time.Now().UTC(),
	}

	if err := db.DB.Create(&transaction).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	successResponse(c, http.StatusOK, gin.H{"message": "Webhook received and balance updated"})
}
