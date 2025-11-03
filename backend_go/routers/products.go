package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterProductRoutes(router *gin.Engine, productHandler *handlers.ProductHandler, authMiddleware *middleware.AuthMiddleware) {
	products := router.Group("/api/products")

	// Группа для роутов, доступных и для пользователей, и для сервисов
	// openAPI := router.Group("/api")
	// {
	products.GET("", productHandler.GetProductsHandler) // TODO: fix auth
	// }

	// Группа для роутов, требующих строгой аутентификации пользователя (JWT)
	products.Use(authMiddleware.RequireAuth)
	{
		products.POST("", middleware.PermissionMiddleware("products:create"), productHandler.CreateProductHandler)
		products.GET("/:id", middleware.PermissionMiddleware("products:read"), productHandler.GetProductHandler)
		products.PATCH("/:id", middleware.PermissionMiddleware("products:update"), productHandler.UpdateProductHandler)
		products.DELETE("/:id", middleware.PermissionMiddleware("products:delete"), productHandler.DeleteProductHandler)
		products.POST("/:id/stock/movements", middleware.PermissionMiddleware("stock:update"), productHandler.CreateStockMovementHandler)
		products.POST("/upload", middleware.PermissionMiddleware("products:create"), productHandler.UploadProductsCSVHandler)
	}
}
