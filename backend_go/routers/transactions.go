package routers

import (
	"net/http"

	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"
	"frbktg/backend_go/responses"

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
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	var response []models.TransactionResponse
	for _, t := range transactions {
		response = append(response, models.TransactionResponse(t))
	}

	responses.SuccessResponse(c, http.StatusOK, response)
}
