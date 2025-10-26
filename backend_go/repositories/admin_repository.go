package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type AdminRepository interface {
	WithTx(tx *gorm.DB) AdminRepository
	GetActiveBotUsers(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.BotUser], error)
	GetBotUserByID(id uint) (*models.BotUser, error)
}

type gormAdminRepository struct {
	db *gorm.DB
}

func NewAdminRepository(db *gorm.DB) AdminRepository {
	return &gormAdminRepository{db: db}
}

func (r *gormAdminRepository) WithTx(tx *gorm.DB) AdminRepository {
	return &gormAdminRepository{db: tx}
}

func (r *gormAdminRepository) GetActiveBotUsers(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.BotUser], error) {
	db := r.db.Model(&models.BotUser{})
	db = ApplyFilters[models.BotUser](db, filters)

	paginatedResult, err := ApplyPagination[models.BotUser](db, page)
	if err != nil {
		return nil, err
	}

	return paginatedResult, nil
}

func (r *gormAdminRepository) GetBotUserByID(id uint) (*models.BotUser, error) {
	var botUser models.BotUser
	if err := r.db.First(&botUser, id).Error; err != nil {
		return nil, err
	}
	return &botUser, nil
}
