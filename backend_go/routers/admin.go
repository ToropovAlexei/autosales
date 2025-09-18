package routers

import (
	"database/sql"
	"errors"
	"net/http"

	"frbktg/backend_go/middleware"
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

func (r *Router) AdminRouter(router *gin.Engine) {
	admin := router.Group("/api/admin")
	admin.Use(middleware.AuthMiddleware(r.appSettings, r.db), middleware.AdminMiddleware())
	admin.GET("/bot-users", r.getBotUsersHandler)
	admin.DELETE("/bot-users/:id", r.deleteBotUserHandler)
}

func (r *Router) getBotUsersHandler(c *gin.Context) {
	var botUsers []models.BotUser
	if err := r.db.Where("is_deleted = ?", false).Find(&botUsers).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	var response []models.BotUserResponse
	for _, u := range botUsers {
		var balance sql.NullFloat64
		if err := r.db.Model(&models.Transaction{}).Where("user_id = ?", u.ID).Select("sum(amount)").
			Row().Scan(&balance); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
			r.logger.Error("Error scanning balance for user", "user_id", u.ID, "error", err)
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

func (r *Router) deleteBotUserHandler(c *gin.Context) {
	var botUser models.BotUser
	if err := r.db.First(&botUser, c.Param("id")).Error; err != nil {
		errorResponse(c, http.StatusNotFound, "Bot user not found")
		return
	}

	botUser.IsDeleted = true
	if err := r.db.Save(&botUser).Error; err != nil {
		errorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}

	c.Status(http.StatusNoContent)
}
