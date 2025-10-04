package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
)

type DashboardHandler struct {
	dashboardService services.DashboardService
}

func NewDashboardHandler(dashboardService services.DashboardService) *DashboardHandler {
	return &DashboardHandler{dashboardService: dashboardService}
}

func (h *DashboardHandler) GetDashboardStatsHandler(c *gin.Context) {
	stats, err := h.dashboardService.GetDashboardStats()
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, stats)
}

func (h *DashboardHandler) GetTimeSeriesDashboardDataHandler(c *gin.Context) {
	startDateStr := c.Query("start_date")
	endDateStr := c.Query("end_date")

	startDate, err := time.Parse(time.RFC3339, startDateStr)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid start_date format"})
		return
	}

	endDate, err := time.Parse(time.RFC3339, endDateStr)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid end_date format"})
		return
	}

	data, err := h.dashboardService.GetTimeSeriesDashboardData(startDate, endDate)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, data)
}

func (h *DashboardHandler) GetTopProductsHandler(c *gin.Context) {
	products, err := h.dashboardService.GetTopProducts()
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, products)
}

func (h *DashboardHandler) GetSalesByCategoryHandler(c *gin.Context) {
	categories, err := h.dashboardService.GetSalesByCategory()
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, categories)
}
