package handlers

import (
	"frbktg/backend_go/apperrors"
	"strconv"

	"github.com/gin-gonic/gin"
)

func getIDFromParam(c *gin.Context) (uint, error) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		return 0, &apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid ID"}
	}
	return uint(id), nil
}

func bindJSON(c *gin.Context, obj interface{}) error {
	if err := c.ShouldBindJSON(obj); err != nil {
		return &apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()}
	}
	return nil
}
