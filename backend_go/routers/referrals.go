package routers

import (
	"frbktg/backend_go/config"
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterStatsRoutes(router *gin.Engine, statsHandler *handlers.StatsHandler, authMiddleware *middleware.AuthMiddleware, appSettings *config.Config) {
	serviceReferrals := router.Group("/api/stats")
	serviceReferrals.Use(middleware.ServiceTokenMiddleware(appSettings))
	{
		serviceReferrals.GET("/referral/:telegram_id", statsHandler.GetReferralStatsHandler)
	}

	adminReferrals := router.Group("/api/stats")
	adminReferrals.Use(authMiddleware.RequireAuth)
	{
		adminReferrals.GET("/referral/admin-list", middleware.PermissionMiddleware("referrals:read"), statsHandler.GetReferralStatsHandler)
	}
}
