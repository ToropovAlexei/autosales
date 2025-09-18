package routers

import (
	"net/http"

	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
)

func (r *Router) TransactionsRouter(router *gin.Engine) {
	auth := router.Group("/api/transactions")
	auth.Use(middleware.AuthMiddleware(r.appSettings, r.db))
	{
		auth.GET("", r.getAllTransactionsHandler)
	}
}

func (r *Router) getAllTransactionsHandler(c *gin.Context) {
	var transactions []models.Transaction
	if err := r.db.Order("created_at desc").Find(&transactions).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	var response []models.TransactionResponse
	for _, t := range transactions {
		response = append(response, models.TransactionResponse(t))
	}

	successResponse(c, http.StatusOK, response)
}
