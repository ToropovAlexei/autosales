package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) DashboardRouter(router *gin.Engine, dashboardHandler *handlers.DashboardHandler) {
	auth := router.Group("/api/dashboard")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.tokenService, r.userRepo))
	{
		auth.GET("/stats", dashboardHandler.GetDashboardStatsHandler)
		auth.GET("/time-series", dashboardHandler.GetTimeSeriesDashboardDataHandler)
		auth.GET("/stats-last-30-days", dashboardHandler.GetDashboardStatsWithTrendHandler)
		auth.GET("/top-products", dashboardHandler.GetTopProductsHandler)
		auth.GET("/sales-by-category", dashboardHandler.GetSalesByCategoryHandler)
	}
}
