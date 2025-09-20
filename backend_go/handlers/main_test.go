package handlers

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

			var notFoundErr *apperrors.ErrNotFound
			var validationErr *apperrors.ErrValidation
			var appErr *apperrors.Error

			switch {
			case errors.As(err, &notFoundErr):
				c.JSON(http.StatusNotFound, gin.H{
					"success": false,
					"error":   notFoundErr.Error(),
				})
			case errors.As(err, &validationErr):
				c.JSON(http.StatusBadRequest, gin.H{
					"success": false,
					"error":   validationErr.Error(),
				})
			case errors.As(err, &appErr):
				c.JSON(appErr.Code, gin.H{
					"success": false,
					"error":   appErr.Message,
				})
			default:
				c.JSON(http.StatusInternalServerError, gin.H{
					"success": false,
					"error":   err.Error(),
				})
			}
		}
	}
}
