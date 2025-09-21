package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) CategoriesRouter(router *gin.Engine, categoryHandler *handlers.CategoryHandler) {
	// Группа для роутов, доступных и для пользователей, и для сервисов
	openAPI := router.Group("/api")
	{
		// Этот роут проверяет или JWT, или сервисный ключ
		openAPI.GET("/categories", middleware.AuthOrServiceTokenMiddleware(r.appSettings, r.tokenService, r.userRepo), categoryHandler.GetCategoriesHandler)
	}

	// Группа для роутов, требующих строгой аутентификации пользователя (JWT)
	authAPI := router.Group("/api/categories")
	authAPI.Use(middleware.AuthMiddleware(r.appSettings, r.tokenService, r.userRepo))
	{
		authAPI.POST("", categoryHandler.CreateCategoryHandler)
		authAPI.GET("/:id", categoryHandler.GetCategoryHandler)
		authAPI.PUT("/:id", categoryHandler.UpdateCategoryHandler)
		authAPI.DELETE("/:id", categoryHandler.DeleteCategoryHandler)
	}
}
