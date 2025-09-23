package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) ReferralsRouter(router *gin.Engine, referralHandler *handlers.ReferralHandler) {
	service := router.Group("/api/referrals")
	service.Use(middleware.ServiceTokenMiddleware(r.appSettings))
	{
		service.POST("", referralHandler.CreateReferralBotHandler)
		service.GET("", referralHandler.GetReferralBotsHandler)
		service.GET("/user/:telegram_id", referralHandler.GetReferralBotsByTelegramIDHandler)
	}

	auth := router.Group("/api/referrals")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.tokenService, r.userRepo))
	{
		auth.GET("/admin-list", referralHandler.GetReferralBotsAdminHandler)
		auth.PUT("/:id/status", referralHandler.UpdateReferralBotStatusHandler)
		auth.PUT("/:id/set-primary", referralHandler.SetPrimaryBotHandler)
		auth.DELETE("/:id", referralHandler.DeleteReferralBotHandler)
	}
}
