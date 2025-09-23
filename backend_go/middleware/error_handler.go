package middleware

import (
	"errors"
	"frbktg/backend_go/apperrors"
	"net/http"

	"github.com/gin-gonic/gin"
)

func ErrorHandler() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Next()

		if len(c.Errors) > 0 {
			err := c.Errors.Last().Err
			logger := GetLogger(c) // Получаем обогащенный логгер

			var notFoundErr *apperrors.ErrNotFound
			var validationErr *apperrors.ErrValidation
			var outOfStockErr *apperrors.ErrOutOfStock
			var alreadyExistsErr *apperrors.ErrAlreadyExists
			var appErr *apperrors.Error

			switch {
			case errors.As(err, &notFoundErr):
				logger.Warn().Err(err).Msg("Not found error")
				c.JSON(http.StatusNotFound, gin.H{
					"success": false,
					"error":   notFoundErr.Error(),
				})
			case errors.As(err, &validationErr):
				logger.Warn().Err(err).Msg("Validation error")
				c.JSON(http.StatusBadRequest, gin.H{
					"success": false,
					"error":   validationErr.Error(),
				})
			case errors.As(err, &outOfStockErr):
				logger.Warn().Err(err).Msg("Out of stock error")
				c.JSON(http.StatusBadRequest, gin.H{
					"success": false,
					"error":   "Product out of stock", // Стабильное сообщение для бота
				})
			case errors.As(err, &alreadyExistsErr):
				logger.Warn().Err(err).Msg("Already exists error")
				c.JSON(http.StatusConflict, gin.H{ // 409 Conflict
					"success": false,
					"error":   alreadyExistsErr.Error(),
				})
			case errors.As(err, &appErr):
				logger.Warn().Err(err).Int("code", appErr.Code).Msg("Application error")
				c.JSON(appErr.Code, gin.H{
					"success": false,
					"error":   appErr.Message,
				})
			default:
				// zerolog автоматически подхватит stack trace из ошибки, обернутой с помощью pkg/errors
				logger.Error().Err(err).Msg("Internal server error")
				c.JSON(http.StatusInternalServerError, gin.H{
					"success": false,
					"error":   "Internal Server Error", // Не показываем детали ошибки пользователю
				})
			}
		}
	}
}
