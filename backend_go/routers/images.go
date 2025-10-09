package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterImageRoutes(router *gin.Engine, imageHandler *handlers.ImageHandler, authMiddleware *middleware.AuthMiddleware) {
	// Public route to serve images
	router.GET("/images/:id", imageHandler.ServeImageHandler)

	// Admin routes for managing images
	adminImagesAPI := router.Group("/api/admin/images")
	adminImagesAPI.Use(authMiddleware.RequireAuth, middleware.AdminMiddleware())
	{
		adminImagesAPI.GET("", imageHandler.ListImagesHandler)
		adminImagesAPI.POST("/upload", imageHandler.UploadImageHandler)
	}
}
