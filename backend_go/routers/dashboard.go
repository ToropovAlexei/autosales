package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) DashboardRouter(router *gin.Engine, dashboardHandler *handlers.DashboardHandler) {
	auth := router.Group("/api/dashboard")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.db))
	{
		auth.GET("/stats", dashboardHandler.GetDashboardStatsHandler)
		auth.GET("/sales-over-time", dashboardHandler.GetSalesOverTimeHandler)
	}
}
