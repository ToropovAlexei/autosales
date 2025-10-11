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
		categories.POST("", middleware.PermissionMiddleware("categories:create"), categoryHandler.CreateCategoryHandler)
		categories.GET("/:id", middleware.PermissionMiddleware("categories:read"), categoryHandler.GetCategoryHandler)
		categories.PUT("/:id", middleware.PermissionMiddleware("categories:update"), categoryHandler.UpdateCategoryHandler)
		categories.DELETE("/:id", middleware.PermissionMiddleware("categories:delete"), categoryHandler.DeleteCategoryHandler)
	}
}
