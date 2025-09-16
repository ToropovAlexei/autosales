package routers

import (
	"net/http"

	"frbktg/backend_go/db"
	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
)

func StockRouter(router *gin.Engine) {
	auth := router.Group("/api/stock")
	auth.Use(middleware.AuthMiddleware())
	{
		auth.GET("/movements", getStockMovementsHandler)
	}
}

func getStockMovementsHandler(c *gin.Context) {
	var movements []models.StockMovement
	if err := db.DB.Order("created_at desc").Find(&movements).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}
	successResponse(c, http.StatusOK, movements)
}
