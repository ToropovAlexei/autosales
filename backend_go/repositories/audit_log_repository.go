package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type AuditLogRepository interface {
	Create(log *models.AuditLog) error
	GetAuditLogs(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.AuditLog], error)
}

type gormAuditLogRepository struct {
	db *gorm.DB
}

func NewAuditLogRepository(db *gorm.DB) AuditLogRepository {
	return &gormAuditLogRepository{db: db}
}

func (r *gormAuditLogRepository) Create(log *models.AuditLog) error {
	return r.db.Create(log).Error
}

func (r *gormAuditLogRepository) GetAuditLogs(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.AuditLog], error) {
	db := r.db.Model(&models.AuditLog{})
	db = ApplyFilters[models.AuditLog](db, filters)
	return ApplyPagination[models.AuditLog](db, page)
}
