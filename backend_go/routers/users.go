package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (rtr *Router) UsersRouter(r *gin.Engine, userHandler *handlers.UserHandler) {
	users := r.Group("/api/users")
	// Service API, not for end-users
	users.Use(middleware.ServiceTokenMiddleware(rtr.appSettings))

	users.POST("/register", userHandler.RegisterBotUserHandler)
	users.GET("/:telegram_id", userHandler.GetBotUserHandler)
	users.GET("/:telegram_id/balance", userHandler.GetBalanceHandler)
	users.GET("/:telegram_id/transactions", userHandler.GetUserTransactionsHandler)
	users.GET("/:telegram_id/subscriptions", userHandler.GetUserSubscriptionsHandler)
	users.GET("/:telegram_id/orders", userHandler.GetUserOrdersHandler)
	users.PUT("/:telegram_id/captcha-status", userHandler.UpdateUserCaptchaStatusHandler)

	// Admin/Seller API
	me := r.Group("/api/me")
	me.Use(middleware.AuthMiddleware(rtr.appSettings, rtr.tokenService, rtr.userRepo))
	me.GET("", userHandler.GetMeHandler)
	me.PUT("/referral-settings", userHandler.UpdateReferralSettingsHandler)

	// Public API
	r.GET("/api/users/seller-settings", userHandler.GetSellerSettingsHandler)
}
