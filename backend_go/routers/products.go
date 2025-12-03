package routers

import (
	"frbktg/backend_go/config"
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterProductRoutes(router *gin.Engine, productHandler *handlers.ProductHandler, authMiddleware *middleware.AuthMiddleware, appSettings *config.Config) {
	// --- Public/Admin Panel Routes ---
	products := router.Group("/api/products")
	{
		// Publicly accessible endpoint for products
		products.GET("", productHandler.GetProductsHandler)

		// Routes for admin panel (JWT auth)
		adminProducts := products.Group("")
		adminProducts.Use(authMiddleware.RequireAuth)
		{
			adminProducts.POST("", middleware.PermissionMiddleware("products:create"), productHandler.CreateProductHandler)
			adminProducts.GET("/:id", middleware.PermissionMiddleware("products:read"), productHandler.GetProductHandler)
			adminProducts.PATCH("/:id", middleware.PermissionMiddleware("products:update"), productHandler.UpdateProductHandler)
			adminProducts.DELETE("/:id", middleware.PermissionMiddleware("products:delete"), productHandler.DeleteProductHandler)
			adminProducts.POST("/:id/stock/movements", middleware.PermissionMiddleware("stock:update"), productHandler.CreateStockMovementHandler)
			adminProducts.POST("/upload", middleware.PermissionMiddleware("products:create"), productHandler.UploadProductsCSVHandler)
		}
	}

	// --- Bot Admin Routes ---
	botAdminProducts := router.Group("/api/bot/admin/products")
	botAdminProducts.Use(authMiddleware.BotAdminAuthMiddleware())
	{
		botAdminProducts.POST("", middleware.PermissionMiddleware("products:create"), productHandler.CreateProductHandler)
		botAdminProducts.PUT("/:id", middleware.PermissionMiddleware("products:update"), productHandler.UpdateProductHandler)
		botAdminProducts.DELETE("/:id", middleware.PermissionMiddleware("products:delete"), productHandler.DeleteProductHandler)
	}
}
