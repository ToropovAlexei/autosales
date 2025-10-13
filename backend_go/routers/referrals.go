package routers

import (
	"frbktg/backend_go/config"
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterReferralRoutes(router *gin.Engine, referralHandler *handlers.ReferralHandler, authMiddleware *middleware.AuthMiddleware, appSettings config.Settings) {
	serviceReferrals := router.Group("/api/referrals")
	serviceReferrals.Use(middleware.ServiceTokenMiddleware(appSettings))
	{
		serviceReferrals.POST("", referralHandler.CreateReferralBotHandler)
		serviceReferrals.GET("", referralHandler.GetReferralBotsHandler)
		serviceReferrals.GET("/user/:telegram_id", referralHandler.GetReferralBotsByTelegramIDHandler)
		serviceReferrals.GET("/stats/:telegram_id", referralHandler.GetReferralStatsHandler)
		serviceReferrals.PUT("/:id/set-primary", referralHandler.ServiceSetPrimaryBotHandler)
		serviceReferrals.DELETE("/:id", referralHandler.ServiceDeleteReferralBotHandler)
	}

	adminReferrals := router.Group("/api/referrals")
	adminReferrals.Use(authMiddleware.RequireAuth)
	{
		adminReferrals.GET("/admin-list", middleware.PermissionMiddleware("referrals:read"), referralHandler.GetAllReferralBotsAdminHandler)
		adminReferrals.PUT("/:id/status", middleware.PermissionMiddleware("referrals:update"), referralHandler.UpdateReferralBotStatusHandler)
		adminReferrals.PUT("/:id/percentage", middleware.PermissionMiddleware("referrals:update"), referralHandler.UpdateReferralBotPercentageHandler)
	}
}
