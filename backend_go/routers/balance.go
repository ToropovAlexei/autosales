package routers

import (
	"net/http"
	"time"

	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
)

func (r *Router) BalanceRouter(router *gin.Engine) {
	service := router.Group("/api/balance")
	service.Use(middleware.ServiceTokenMiddleware(r.appSettings))
	service.POST("/deposit", r.depositBalanceHandler)

	router.POST("/api/balance/webhook", r.paymentWebhookHandler)
}

type DepositRequest struct {
	UserID int64   `json:"user_id"`
	Amount float64 `json:"amount"`
}

func (r *Router) updateBalance(c *gin.Context, userID int64, amount float64, description string) bool {
	var user models.BotUser
	if err := r.db.Where("telegram_id = ? AND is_deleted = ?", userID, false).First(&user).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return false
	}

	transaction := models.Transaction{
		UserID:      user.ID,
		Type:        models.Deposit,
		Amount:      amount,
		Description: description,
		CreatedAt:   time.Now().UTC(),
	}

	if err := r.db.Create(&transaction).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return false
	}
	return true
}

func (r *Router) depositBalanceHandler(c *gin.Context) {
	var json DepositRequest
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	if r.updateBalance(c, json.UserID, json.Amount, "Test deposit") {
		successResponse(c, http.StatusOK, gin.H{"message": "Balance updated successfully"})
	}
}

type WebhookPayload struct {
	UserID int64   `json:"user_id"`
	Amount float64 `json:"amount"`
}

func (r *Router) paymentWebhookHandler(c *gin.Context) {
	var json WebhookPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		errorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	if r.updateBalance(c, json.UserID, json.Amount, "Deposit via webhook") {
		successResponse(c, http.StatusOK, gin.H{"message": "Webhook received and balance updated"})
	}
}
