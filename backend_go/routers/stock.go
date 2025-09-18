package routers

import (
	"net/http"

	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"
	"frbktg/backend_go/responses"

	"github.com/gin-gonic/gin"
)

func (r *Router) StockRouter(router *gin.Engine) {
	auth := router.Group("/api/stock")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.db))
	{
		auth.GET("/movements", r.getStockMovementsHandler)
	}
}

func (r *Router) getStockMovementsHandler(c *gin.Context) {
	var movements []models.StockMovement
	if err := r.db.Order("created_at desc").Find(&movements).Error; err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
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

	responses.SuccessResponse(c, http.StatusOK, response)
}
