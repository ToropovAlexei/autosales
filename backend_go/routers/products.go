package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func (r *Router) ProductsRouter(router *gin.Engine, productHandler *handlers.ProductHandler) {
	auth := router.Group("/api/products")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.db))
	{
		auth.GET("", productHandler.GetProductsHandler)
		auth.POST("", productHandler.CreateProductHandler)
		auth.GET("/:id", productHandler.GetProductHandler)
		auth.PUT("/:id", productHandler.UpdateProductHandler)
		auth.DELETE("/:id", productHandler.DeleteProductHandler)
		auth.POST("/:id/stock/movements", productHandler.CreateStockMovementHandler)
	}
}
