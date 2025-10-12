package services

import (
	"encoding/json"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"github.com/gin-gonic/gin"
	"gorm.io/datatypes"
)

type AuditLogService interface {
	Log(ctx *gin.Context, action string, targetType string, targetID uint, changes map[string]interface{}) error
	GetAuditLogs(page, pageSize int) ([]models.AuditLog, int64, error)
}

type auditLogService struct {
	repo repositories.AuditLogRepository
}

func NewAuditLogService(repo repositories.AuditLogRepository) AuditLogService {
	return &auditLogService{repo: repo}
}

func (s *auditLogService) Log(ctx *gin.Context, action string, targetType string, targetID uint, changes map[string]interface{}) error {
	user, _ := ctx.Get("user")
	currentUser, _ := user.(models.User)

	changesJSON, err := json.Marshal(changes)
	if err != nil {
		return err
	}

	logEntry := models.AuditLog{
		UserID:     currentUser.ID,
		UserEmail:  currentUser.Email,
		Action:     action,
		TargetType: targetType,
		TargetID:   targetID,
		Changes:    datatypes.JSON(changesJSON),
		Status:     "SUCCESS", // Assuming success for now, can be enhanced later
		IPAddress:  ctx.ClientIP(),
		UserAgent:  ctx.Request.UserAgent(),
	}

	return s.repo.Create(&logEntry)
}

func (s *auditLogService) GetAuditLogs(page, pageSize int) ([]models.AuditLog, int64, error) {
	return s.repo.GetPaginated(page, pageSize)
}
