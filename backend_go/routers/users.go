package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (rtr *Router) UsersRouter(r *gin.Engine, userHandler *handlers.UserHandler) {
	// Service API, not for end-users, used by the bot
	serviceAPI := r.Group("/api/users")
	serviceAPI.Use(middleware.ServiceTokenMiddleware(rtr.appSettings))
	{
		serviceAPI.POST("/register", userHandler.RegisterBotUserHandler)
		serviceAPI.GET("/:telegram_id", userHandler.GetBotUserHandler)
		serviceAPI.GET("/:telegram_id/balance", userHandler.GetBalanceHandler)
		serviceAPI.GET("/:telegram_id/transactions", userHandler.GetUserTransactionsHandler)
		serviceAPI.GET("/:telegram_id/subscriptions", userHandler.GetUserSubscriptionsHandler)
		serviceAPI.GET("/:telegram_id/orders", userHandler.GetUserOrdersHandler)
		serviceAPI.PUT("/:telegram_id/captcha-status", userHandler.UpdateUserCaptchaStatusHandler)
	}

	// Admin API for managing bot users
	adminBotUsersAPI := r.Group("/api/admin/bot-users")
	adminBotUsersAPI.Use(middleware.AuthMiddleware(rtr.appSettings, rtr.tokenService, rtr.userRepo))
	{
		adminBotUsersAPI.PATCH("/:telegram_id/toggle-block", userHandler.ToggleBlockUserHandler)
	}

	// Admin/Seller API for their own data
	me := r.Group("/api/me")
	me.Use(middleware.AuthMiddleware(rtr.appSettings, rtr.tokenService, rtr.userRepo))
	{
		me.GET("", userHandler.GetMeHandler)
		me.PUT("/referral-settings", userHandler.UpdateReferralSettingsHandler)
	}

	// Public API
	r.GET("/api/users/seller-settings", userHandler.GetSellerSettingsHandler)
}
