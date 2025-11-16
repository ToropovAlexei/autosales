package handlers

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"github.com/gin-gonic/gin"
	"net/http"
)

type StoreBalanceHandler struct {
	service services.StoreBalanceService
}

func NewStoreBalanceHandler(service services.StoreBalanceService) *StoreBalanceHandler {
	return &StoreBalanceHandler{service: service}
}

// GetStoreBalance godoc
// @Summary Get store balance
// @Description Get current store balance
// @Tags Admin
// @Accept  json
// @Produce  json
// @Success 200 {object} responses.ResponseSchema[models.StoreBalanceResponse]
// @Failure 500 {object} responses.ErrorResponse
// @Router /admin/store-balance [get]
func (h *StoreBalanceHandler) GetStoreBalance(c *gin.Context) {
	balance, err := h.service.GetStoreBalance()
	if err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, "Failed to get store balance")
		return
	}

	responses.SuccessResponse(c, http.StatusOK, models.StoreBalanceResponse{CurrentBalance: balance.CurrentBalance})
}
