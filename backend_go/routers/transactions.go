package routers

import (
	"net/http"

	"frbktg/backend_go/db"
	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"
	

	"github.com/gin-gonic/gin"
)

func TransactionsRouter(router *gin.Engine) {
	auth := router.Group("/api/transactions")
	auth.Use(middleware.AuthMiddleware())
	{
		auth.GET("", getAllTransactionsHandler)
	}
}

func getAllTransactionsHandler(c *gin.Context) {
	var transactions []models.Transaction
	if err := db.DB.Order("created_at desc").Find(&transactions).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	var response []models.TransactionResponse
	for _, t := range transactions {
		response = append(response, models.TransactionResponse{
			ID:          t.ID,
			UserID:      t.UserID,
			OrderID:     t.OrderID,
			Type:        t.Type,
			Amount:      t.Amount,
			CreatedAt:   t.CreatedAt,
			Description: t.Description,
		})
	}

	successResponse(c, http.StatusOK, response)
}
