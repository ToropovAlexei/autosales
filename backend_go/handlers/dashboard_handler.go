package handlers

import (
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
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}
	responses.SuccessResponse(c, http.StatusOK, stats)
}

func (h *DashboardHandler) GetSalesOverTimeHandler(c *gin.Context) {
	startDateStr := c.Query("start_date")
	endDateStr := c.Query("end_date")

	startDate, err := time.Parse(time.RFC3339, startDateStr)
	if err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, "Invalid start_date format")
		return
	}

	endDate, err := time.Parse(time.RFC3339, endDateStr)
	if err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, "Invalid end_date format")
		return
	}

	salesData, err := h.dashboardService.GetSalesOverTime(startDate, endDate)
	if err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	responses.SuccessResponse(c, http.StatusOK, salesData)
}
