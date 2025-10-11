package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterImageRoutes(router *gin.Engine, imageHandler *handlers.ImageHandler, authMiddleware *middleware.AuthMiddleware) {
	images := router.Group("/api/images")

	// Public route to get an image
	images.GET("/:id", imageHandler.ServeImageHandler)

	// Protected routes for uploading and deleting images
	images.Use(authMiddleware.RequireAuth)
	images.Use(middleware.PermissionMiddleware("products:update")) // Example permission
	{
		images.POST("", imageHandler.UploadImageHandler)
		images.DELETE("/:id", imageHandler.DeleteImageHandler)
	}
}
