package handlers

import (
	"frbktg/backend_go/apperrors"
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

// @Summary      (DEPRECATED) Deposit Balance
// @Description  Manually adds a deposit transaction to a user's balance. Prefer using the /deposit/invoice flow.
// @Tags         Balance
// @Accept       json
// @Produce      json
// @Param        deposit body depositRequest true "Deposit data"
// @Success      200 {object} responses.ResponseSchema[responses.MessageResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      404 {object} responses.ErrorResponseSchema
// @Router       /balance/deposit [post]
// @Security     ServiceApiKeyAuth
func (h *BalanceHandler) DepositBalanceHandler(c *gin.Context) {
	var json depositRequest
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	if err := h.balanceService.DepositBalance(json.UserID, json.Amount, "Test deposit"); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, responses.MessageResponse{Message: "Balance updated successfully"})
}

type webhookPayload struct {
	UserID int64   `json:"user_id" binding:"required"`
	Amount float64 `json:"amount" binding:"required,gt=0"`
}

func (h *BalanceHandler) PaymentWebhookHandler(c *gin.Context) {
	var json webhookPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	if err := h.balanceService.DepositBalance(json.UserID, json.Amount, "Deposit via webhook"); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{"message": "Webhook received and balance updated"})
}