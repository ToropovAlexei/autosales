package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterAdminReferralRoutes(router *gin.Engine, botHandler *handlers.BotHandler, authMiddleware *middleware.AuthMiddleware) {
	adminReferrals := router.Group("/api/admin/referral-bots")
	adminReferrals.Use(authMiddleware.RequireAuth)
	{
		adminReferrals.GET("", middleware.PermissionMiddleware("referrals:read"), botHandler.GetReferralBotsAdminHandler)
		adminReferrals.DELETE("/:id", middleware.PermissionMiddleware("referrals:write"), botHandler.DeleteBotHandler)
		adminReferrals.PATCH("/:id/status", middleware.PermissionMiddleware("referrals:write"), botHandler.UpdateBotStatusHandler)
		adminReferrals.PUT("/:id/set-primary", middleware.PermissionMiddleware("referrals:write"), botHandler.SetPrimaryBotHandler)
		adminReferrals.PUT("/:id/percentage", middleware.PermissionMiddleware("referrals:write"), botHandler.UpdateBotReferralPercentageHandler)
	}
}
