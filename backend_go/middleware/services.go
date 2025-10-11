package middleware

import (
	"frbktg/backend_go/services"

	"github.com/gin-gonic/gin"
)

func ServicesMiddleware(roleService services.RoleService) gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Set("roleService", roleService)
		c.Next()
	}
}
