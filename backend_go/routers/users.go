package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) UsersRouter(router *gin.Engine, userHandler *handlers.UserHandler) {
	auth := router.Group("/api/users")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.tokenService, r.userRepo))
	{
		auth.GET("/me", userHandler.GetMeHandler)
		auth.PUT("/me/referral-settings", userHandler.UpdateReferralSettingsHandler)
	}

	service := router.Group("/api/users")
	service.Use(middleware.ServiceTokenMiddleware(r.appSettings))
	{
		service.POST("/register", userHandler.RegisterBotUserHandler)
		service.GET("/:id", userHandler.GetBotUserHandler)
		service.GET("/:id/balance", userHandler.GetBalanceHandler)
		service.GET("/:id/transactions", userHandler.GetUserTransactionsHandler)
		service.PUT("/:id/captcha-status", userHandler.UpdateUserCaptchaStatusHandler)
		service.GET("/seller-settings", userHandler.GetSellerSettingsHandler)
	}
}
