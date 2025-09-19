package handlers

import (
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"

	"github.com/gin-gonic/gin"
)

type StockHandler struct {
	stockService services.StockService
}

func NewStockHandler(stockService services.StockService) *StockHandler {
	return &StockHandler{stockService: stockService}
}

func (h *StockHandler) GetStockMovementsHandler(c *gin.Context) {
	movements, err := h.stockService.GetStockMovements()
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, movements)
}