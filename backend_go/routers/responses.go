package routers

import (
	"github.com/gin-gonic/gin"
)

func successResponse(c *gin.Context, statusCode int, data interface{}) {
	c.JSON(statusCode, gin.H{
		"success": true,
		"data":    data,
		"error":   nil,
	})
}

func errorResponse(c *gin.Context, statusCode int, err string) {
	c.JSON(statusCode, gin.H{
		"success": false,
		"data":    nil,
		"error":   err,
	})
}
