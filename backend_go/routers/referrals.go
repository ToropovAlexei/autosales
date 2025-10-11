package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterReferralRoutes(router *gin.Engine, referralHandler *handlers.ReferralHandler, authMiddleware *middleware.AuthMiddleware) {
	referrals := router.Group("/api/referrals")

	// service.Use(middleware.ServiceTokenMiddleware(r.appSettings))
	{
		referrals.POST("", referralHandler.CreateReferralBotHandler)
		referrals.GET("", referralHandler.GetReferralBotsHandler)
		referrals.GET("/user/:telegram_id", referralHandler.GetReferralBotsByTelegramIDHandler)
		referrals.PUT("/:id/set-primary", referralHandler.ServiceSetPrimaryBotHandler)
		referrals.DELETE("/:id", referralHandler.ServiceDeleteReferralBotHandler)
	}

	referrals.Use(authMiddleware.RequireAuth)
	{
		referrals.GET("/admin-list", middleware.PermissionMiddleware("referrals:read"), referralHandler.GetAllReferralBotsAdminHandler)
		referrals.PUT("/:id/status", middleware.PermissionMiddleware("referrals:update"), referralHandler.UpdateReferralBotStatusHandler)
		referrals.PUT("/:id/percentage", middleware.PermissionMiddleware("referrals:update"), referralHandler.UpdateReferralBotPercentageHandler)
	}
}
