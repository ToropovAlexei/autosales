package responses

import (
	"github.com/gin-gonic/gin"
)

// ResponseSchema defines the generic structure for API responses.
// @Description This is the standard API response structure.
type ResponseSchema[T any] struct {
	Success bool    `json:"success" example:"true"`
	Data    *T      `json:"data,omitempty"`
	Error   *string `json:"error,omitempty" extensions:"x-nullable=true" example:"null"`
}

// ErrorResponseSchema defines the structure for a failed API response, used for documentation.
// @Description This is the standard error response structure.
type ErrorResponseSchema struct {
	Success bool         `json:"success" example:"false"`
	Data    *interface{} `json:"data"`
	Error   *string      `json:"error"`
}

func ErrorResponse(c *gin.Context, statusCode int, message string) {
	c.JSON(statusCode, gin.H{"success": false, "data": nil, "error": message})
}

func SuccessResponse(c *gin.Context, statusCode int, data interface{}) {
	c.JSON(statusCode, gin.H{"success": true, "data": data, "error": nil})
}
