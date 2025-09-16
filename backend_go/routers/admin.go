package routers

import (
	"net/http"

	"frbktg/backend_go/db"
	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"
	"frbktg/backend_go/models/responses"

	"github.com/gin-gonic/gin"
)

func AdminRouter(router *gin.Engine) {
	admin := router.Group("/api/admin")
	admin.Use(middleware.AuthMiddleware(), middleware.AdminMiddleware())
	{
		admin.GET("/bot-users", getBotUsersHandler)
		admin.DELETE("/bot-users/:id", deleteBotUserHandler)
	}
}

func getBotUsersHandler(c *gin.Context) {
	var botUsers []models.BotUser
	if err := db.DB.Where("is_deleted = ?", false).Find(&botUsers).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	var response []responses.BotUserResponse
	for _, u := range botUsers {
		var balance float64
		db.DB.Model(&models.Transaction{}).Where("user_id = ?", u.ID).Select("sum(amount)").Row().Scan(&balance)
		response = append(response, responses.BotUserResponse{
			ID:               u.ID,
			TelegramID:       u.TelegramID,
			IsDeleted:        u.IsDeleted,
			HasPassedCaptcha: u.HasPassedCaptcha,
			Balance:          balance,
		})
	}

	successResponse(c, http.StatusOK, response)
}

func deleteBotUserHandler(c *gin.Context) {
	var botUser models.BotUser
	if err := db.DB.First(&botUser, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return
	}

	botUser.IsDeleted = true
	if err := db.DB.Save(&botUser).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	c.Status(http.StatusNoContent)
}
