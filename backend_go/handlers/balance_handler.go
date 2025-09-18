package handlers

import (
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"

	"github.com/gin-gonic/gin"
)

type BalanceHandler struct {
	balanceService services.BalanceService
}

func NewBalanceHandler(balanceService services.BalanceService) *BalanceHandler {
	return &BalanceHandler{balanceService: balanceService}
}

type depositRequest struct {
	UserID int64   `json:"user_id" binding:"required"`
	Amount float64 `json:"amount" binding:"required,gt=0"`
}

func (h *BalanceHandler) DepositBalanceHandler(c *gin.Context) {
	var json depositRequest
	if err := c.ShouldBindJSON(&json); err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	if err := h.balanceService.DepositBalance(json.UserID, json.Amount, "Test deposit"); err != nil {
		responses.ErrorResponse(c, http.StatusNotFound, err.Error())
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{"message": "Balance updated successfully"})
}

type webhookPayload struct {
	UserID int64   `json:"user_id" binding:"required"`
	Amount float64 `json:"amount" binding:"required,gt=0"`
}

func (h *BalanceHandler) PaymentWebhookHandler(c *gin.Context) {
	var json webhookPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	if err := h.balanceService.DepositBalance(json.UserID, json.Amount, "Deposit via webhook"); err != nil {
		responses.ErrorResponse(c, http.StatusNotFound, err.Error())
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{"message": "Webhook received and balance updated"})
}
