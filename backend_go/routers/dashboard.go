package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterDashboardRoutes(router *gin.Engine, dashboardHandler *handlers.DashboardHandler, authMiddleware *middleware.AuthMiddleware) {
	dashboard := router.Group("/api/dashboard")
	dashboard.Use(authMiddleware.RequireAuth)
	dashboard.Use(middleware.PermissionMiddleware("dashboard:read"))
	{
		dashboard.GET("/stats", dashboardHandler.GetDashboardStatsHandler)
		dashboard.GET("/time-series", dashboardHandler.GetTimeSeriesDashboardDataHandler)
		dashboard.GET("/top-products", dashboardHandler.GetTopProductsHandler)
		dashboard.GET("/sales-by-category", dashboardHandler.GetSalesByCategoryHandler)
	}
}
