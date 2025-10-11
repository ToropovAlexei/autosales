package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterAdminUserRoutes(router *gin.Engine, roleHandler *handlers.RoleHandler, adminHandler *handlers.AdminHandler, authMiddleware *middleware.AuthMiddleware) {
	users := router.Group("/api/admin/users")
	users.Use(authMiddleware.RequireAuth)
	users.Use(middleware.PermissionMiddleware("rbac:manage"))
	{
		users.GET("", adminHandler.GetUsersHandler)
		users.GET("/:id/roles", roleHandler.GetUserRolesHandler)
		users.PUT("/:id/roles", roleHandler.SetUserRoleHandler)
		users.GET("/:id/permissions", roleHandler.GetUserPermissionsHandler)
		users.POST("/:id/permissions", roleHandler.AddUserPermissionHandler)
		users.DELETE("/:id/permissions/:permission_id", roleHandler.RemoveUserPermissionHandler)
	}
}
