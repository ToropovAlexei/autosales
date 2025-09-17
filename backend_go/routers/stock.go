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

	var response []models.StockMovementResponse
	for _, m := range movements {
		response = append(response, models.StockMovementResponse{
			ID:          m.ID,
			ProductID:   m.ProductID,
			Type:        m.Type,
			Quantity:    m.Quantity,
			CreatedAt:   m.CreatedAt,
			Description: m.Description,
			OrderID:     m.OrderID,
		})
	}

	successResponse(c, http.StatusOK, response)
}
