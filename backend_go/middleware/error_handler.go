package middleware

import (
	"errors"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"net/http"

	"github.com/gin-gonic/gin"
)

func ErrorHandler() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Next()

		if len(c.Errors) > 0 {
			err := c.Errors.Last().Err

			var notFound *apperrors.ErrNotFound
			var validation *apperrors.ErrValidation
			var insufficientBalance *apperrors.ErrInsufficientBalance
			var outOfStock *apperrors.ErrOutOfStock
			var alreadyExists *apperrors.ErrAlreadyExists
			var forbidden *apperrors.ErrForbidden

			switch {
			case errors.As(err, &notFound):
				responses.ErrorResponse(c, http.StatusNotFound, err.Error())
			case errors.As(err, &validation):
				responses.ErrorResponse(c, http.StatusBadRequest, err.Error())
			case errors.As(err, &insufficientBalance):
				responses.ErrorResponse(c, http.StatusBadRequest, err.Error())
			case errors.As(err, &outOfStock):
				responses.ErrorResponse(c, http.StatusBadRequest, err.Error())
			case errors.As(err, &alreadyExists):
				responses.ErrorResponse(c, http.StatusConflict, err.Error())
			case errors.As(err, &forbidden):
				responses.ErrorResponse(c, http.StatusForbidden, err.Error())
			default:
				responses.ErrorResponse(c, http.StatusInternalServerError, "Internal Server Error")
			}
		}
	}
}
