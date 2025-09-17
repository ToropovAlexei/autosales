package routers

import (
	"net/http"
	"time"

	"frbktg/backend_go/db"
	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

func DashboardRouter(router *gin.Engine) {
	auth := router.Group("/api/dashboard")
	auth.Use(middleware.AuthMiddleware())
	{
		auth.GET("/stats", getDashboardStatsHandler)
		auth.GET("/sales-over-time", getSalesOverTimeHandler)
	}
}

type DashboardStats struct {
	TotalUsers         int64 `json:"total_users"`
	UsersWithPurchases int64 `json:"users_with_purchases"`
	AvailableProducts  int64 `json:"available_products"`
}

func getDashboardStatsHandler(c *gin.Context) {
	var totalUsers int64
	db.DB.Model(&models.BotUser{}).Where("is_deleted = ?", false).Count(&totalUsers)

	var usersWithPurchases int64
	db.DB.Model(&models.Order{}).Distinct("user_id").Count(&usersWithPurchases)

	var productIDs []uint
	db.DB.Model(&models.Product{}).Pluck("id", &productIDs)

	var availableProducts int64
	for _, id := range productIDs {
		var stock int64
		if err := db.DB.Model(&models.StockMovement{}).Where("product_id = ?", id).Select("sum(quantity)").Row().Scan(&stock); err != nil && err != gorm.ErrRecordNotFound {
			errorResponse(c, http.StatusInternalServerError, err.Error())
			return
		}
		if stock > 0 {
			availableProducts++
		}
	}

	stats := DashboardStats{
		TotalUsers:         totalUsers,
		UsersWithPurchases: usersWithPurchases,
		AvailableProducts:  availableProducts,
	}

	successResponse(c, http.StatusOK, stats)
}

type SalesOverTime struct {
	ProductsSold int64   `json:"products_sold"`
	TotalRevenue float64 `json:"total_revenue"`
}

func getSalesOverTimeHandler(c *gin.Context) {
	startDateStr := c.Query("start_date")
	endDateStr := c.Query("end_date")

	startDate, err := time.Parse(time.RFC3339, startDateStr)
	if err != nil {
		errorResponse(c, http.StatusBadRequest, "Invalid start_date")
		return
	}

	endDate, err := time.Parse(time.RFC3339, endDateStr)
	if err != nil {
		errorResponse(c, http.StatusBadRequest, "Invalid end_date")
		return
	}

	var productsSold int64
	db.DB.Model(&models.Order{}).Where("created_at >= ? AND created_at < ?", startDate, endDate.AddDate(0, 0, 1)).Count(&productsSold)

	var totalRevenue float64
	if err := db.DB.Model(&models.Order{}).Where("created_at >= ? AND created_at < ?", startDate, endDate.AddDate(0, 0, 1)).Select("sum(amount)").Row().Scan(&totalRevenue); err != nil && err != gorm.ErrRecordNotFound {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	salesData := SalesOverTime{
		ProductsSold: productsSold,
		TotalRevenue: totalRevenue,
	}

	successResponse(c, http.StatusOK, salesData)
}
