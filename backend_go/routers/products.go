package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) ProductsRouter(router *gin.Engine, productHandler *handlers.ProductHandler) {
	// Группа для роутов, доступных и для пользователей, и для сервисов
	openAPI := router.Group("/api")
	{
		openAPI.GET("/products", middleware.AuthOrServiceTokenMiddleware(r.appSettings, r.tokenService, r.userRepo), productHandler.GetProductsHandler)
	}

	// Группа для роутов, требующих строгой аутентификации пользователя (JWT)
	authAPI := router.Group("/api/products")
	authAPI.Use(middleware.AuthMiddleware(r.appSettings, r.tokenService, r.userRepo))
	{
		authAPI.POST("", productHandler.CreateProductHandler)
		authAPI.GET("/:id", productHandler.GetProductHandler)
		authAPI.PUT("/:id", productHandler.UpdateProductHandler)
		authAPI.DELETE("/:id", productHandler.DeleteProductHandler)
		authAPI.POST("/:id/stock/movements", productHandler.CreateStockMovementHandler)
	}
}
