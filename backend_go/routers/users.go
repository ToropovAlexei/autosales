package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterUserRoutes(router *gin.Engine, userHandler *handlers.UserHandler, authMiddleware *middleware.AuthMiddleware) {
	users := router.Group("/api/users")

	// Service API, not for end-users, used by the bot
	// serviceAPI := users.Group("/")
	// serviceAPI.Use(middleware.ServiceTokenMiddleware(rtr.appSettings))
	{
		users.POST("/register", userHandler.RegisterBotUserHandler)
		users.GET("/:telegram_id", userHandler.GetBotUserHandler)
		users.GET("/:telegram_id/balance", userHandler.GetBalanceHandler)
		users.GET("/:telegram_id/transactions", userHandler.GetUserTransactionsHandler)
		users.GET("/:telegram_id/subscriptions", userHandler.GetUserSubscriptionsHandler)
		users.GET("/:telegram_id/orders", userHandler.GetUserOrdersHandler)
		users.PUT("/:telegram_id/captcha-status", userHandler.UpdateUserCaptchaStatusHandler)
	}


	// Admin/Seller API for their own data
	me := router.Group("/api/me")
	me.Use(authMiddleware.RequireAuth)
	{
		me.GET("", userHandler.GetMeHandler)
		me.PUT("/referral-settings", userHandler.UpdateReferralSettingsHandler)
	}

	// Public API
	// router.GET("/users/seller-settings", userHandler.GetSellerSettingsHandler)
}
