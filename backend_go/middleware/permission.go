package middleware

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/services"

	"github.com/gin-gonic/gin"
)

func PermissionMiddleware(requiredPermission string) gin.HandlerFunc {
	return func(c *gin.Context) {
		roleService, exists := c.Get("roleService")
		if !exists {
			c.Error(apperrors.New(500, "RoleService not found in context", nil))
			c.Abort()
			return
		}

		rs, ok := roleService.(services.RoleService)
		if !ok {
			c.Error(apperrors.New(500, "Invalid RoleService type in context", nil))
			c.Abort()
			return
		}

		user, exists := c.Get("user")
		if !exists {
			c.Error(apperrors.ErrForbidden)
			c.Abort()
			return
		}

		currentUser, ok := user.(models.User)
		if !ok {
			c.Error(apperrors.ErrForbidden)
			c.Abort()
			return
		}

		// Check for super admin
		for _, role := range currentUser.Roles {
			if role.IsSuper {
				c.Next()
				return
			}
		}

		permissions, err := rs.GetUserFinalPermissions(currentUser.ID)
		if err != nil {
			c.Error(err)
			c.Abort()
			return
		}

		hasPermission := false
		for _, p := range permissions {
			if p == requiredPermission {
				hasPermission = true
				break
			}
		}

		if !hasPermission {
			c.Error(apperrors.ErrForbidden)
			c.Abort()
			return
		}

		c.Next()
	}
}
