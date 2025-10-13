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
	adminImages := router.Group("/api/admin/images")
	adminImages.Use(authMiddleware.RequireAuth)
	{
		adminImages.GET("", middleware.PermissionMiddleware("images:read"), imageHandler.ListImagesHandler)
		adminImages.POST("", middleware.PermissionMiddleware("images:upload"), imageHandler.UploadImageHandler)
		adminImages.DELETE("/:id", middleware.PermissionMiddleware("images:delete"), imageHandler.DeleteImageHandler)
	}
}
