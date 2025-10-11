package routers

import (
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func RegisterRoleRoutes(router *gin.Engine, roleHandler *handlers.RoleHandler, authMiddleware *middleware.AuthMiddleware) {
	roles := router.Group("/api/admin/roles")
	roles.Use(authMiddleware.RequireAuth)
	roles.Use(middleware.PermissionMiddleware("rbac:manage"))
	{
		roles.POST("", roleHandler.CreateRoleHandler)
		roles.GET("", roleHandler.GetRolesHandler)
		roles.GET("/:id", roleHandler.GetRoleHandler)
		roles.PUT("/:id", roleHandler.UpdateRoleHandler)
		roles.DELETE("/:id", roleHandler.DeleteRoleHandler)

		roles.GET("/:id/permissions", roleHandler.GetRolePermissionsHandler)
		roles.POST("/:id/permissions", roleHandler.AddPermissionToRoleHandler)
		roles.DELETE("/:id/permissions/:permission_id", roleHandler.RemovePermissionFromRoleHandler)
	}

	permissions := router.Group("/api/admin/permissions")
	permissions.Use(authMiddleware.RequireAuth)
	permissions.Use(middleware.PermissionMiddleware("rbac:manage"))
	{
		permissions.GET("", roleHandler.GetPermissionsHandler)
	}
}
