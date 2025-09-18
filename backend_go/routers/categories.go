package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) CategoriesRouter(router *gin.Engine, categoryHandler *handlers.CategoryHandler) {
	api := router.Group("/api/categories")
	api.Use(middleware.AuthMiddleware(r.appSettings, r.db))
	api.GET("", categoryHandler.GetCategoriesHandler)
	api.POST("", categoryHandler.CreateCategoryHandler)
	api.GET("/:id", categoryHandler.GetCategoryHandler)
	api.PUT("/:id", categoryHandler.UpdateCategoryHandler)
	api.DELETE("/:id", categoryHandler.DeleteCategoryHandler)
}
