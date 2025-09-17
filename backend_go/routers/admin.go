package routers

import (
	"database/sql"
	"log"
	"net/http"

	"frbktg/backend_go/db"
	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

func AdminRouter(router *gin.Engine) {
	admin := router.Group("/api/admin")
	admin.Use(middleware.AuthMiddleware(), middleware.AdminMiddleware())
	admin.GET("/bot-users", getBotUsersHandler)
	admin.DELETE("/bot-users/:id", deleteBotUserHandler)
}

func getBotUsersHandler(c *gin.Context) {
	var botUsers []models.BotUser
	if err := db.DB.Where("is_deleted = ?", false).Find(&botUsers).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	var response []models.BotUserResponse
	for _, u := range botUsers {
		var balance sql.NullFloat64
		if err := db.DB.Model(&models.Transaction{}).Where("user_id = ?", u.ID).Select("sum(amount)").Row().Scan(&balance); err != nil && err != gorm.ErrRecordNotFound {
			log.Printf("Error scanning balance for user %d: %v", u.ID, err)
			// Continue processing, balance will be 0.0 if null or error
		}

		var userBalance float64
		if balance.Valid {
			userBalance = balance.Float64
		}

		response = append(response, models.BotUserResponse{
			ID:               u.ID,
			TelegramID:       u.TelegramID,
			IsDeleted:        u.IsDeleted,
			HasPassedCaptcha: u.HasPassedCaptcha,
			Balance:          userBalance,
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
