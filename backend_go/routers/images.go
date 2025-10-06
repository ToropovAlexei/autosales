package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) ImagesRouter(router *gin.Engine, imageHandler *handlers.ImageHandler) {
	// Public route to serve images
	router.GET("/images/:id", imageHandler.ServeImageHandler)

	// Admin routes for managing images
	adminImagesAPI := router.Group("/api/admin/images")
	adminImagesAPI.Use(middleware.AuthMiddleware(r.appSettings, r.tokenService, r.userRepo))
	adminImagesAPI.Use(middleware.AdminMiddleware())
	{
		adminImagesAPI.GET("", imageHandler.ListImagesHandler)
		adminImagesAPI.POST("/upload", imageHandler.UploadImageHandler)
	}
}
