package routers

import (
	"frbktg/backend_go/di"
	"frbktg/backend_go/handlers"
	"frbktg/backend_go/middleware"

	"github.com/gin-gonic/gin"
)

func SetupAuditLogRoutes(router *gin.RouterGroup, c *di.Container) {
	auditLogHandler := handlers.NewAuditLogHandler(c.AuditLogService)
	authMiddleware := middleware.NewAuthMiddleware(c.TokenService, c.UserService)

	adminRoutes := router.Group("/admin")
	adminRoutes.Use(authMiddleware.RequireAuth, middleware.PermissionMiddleware("audit_log.read"))
	{
		adminRoutes.GET("/audit-logs", auditLogHandler.GetAuditLogs)
	}
}
