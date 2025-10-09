package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterCategoryRoutes(router *gin.Engine, categoryHandler *handlers.CategoryHandler, authMiddleware *middleware.AuthMiddleware) {
	categories := router.Group("/api/categories")
	categories.GET("", categoryHandler.GetCategoriesHandler) // TODO: add auth

	// Группа для роутов, требующих строгой аутентификации пользователя (JWT)
	categories.Use(authMiddleware.RequireAuth)
	{
		categories.POST("", categoryHandler.CreateCategoryHandler)
		categories.GET("/:id", categoryHandler.GetCategoryHandler)
		categories.PUT("/:id", categoryHandler.UpdateCategoryHandler)
		categories.DELETE("/:id", categoryHandler.DeleteCategoryHandler)
	}
}
