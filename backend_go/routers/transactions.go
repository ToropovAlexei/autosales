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
	successResponse(c, http.StatusOK, transactions)
}
