package handlers

import (
	"encoding/json"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
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
	var filters []models.Filter
	if filtersJSON := c.Query("filters"); filtersJSON != "" {
		if err := json.Unmarshal([]byte(filtersJSON), &filters); err != nil {
			c.Error(&apperrors.ErrValidation{Message: "Invalid filters format"})
			return
		}
	}

	for i, f := range filters {
		if f.Field == "product_id" {
			if v, ok := f.Value.(float64); ok {
				filters[i].Value = uint(v)
			}
		}
	}

	movements, err := h.stockService.GetStockMovements(filters)
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, movements)
}
