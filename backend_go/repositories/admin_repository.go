package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type AdminRepository interface {
	GetActiveBotUsers() ([]models.BotUser, error)
	GetBotUserByID(id uint) (*models.BotUser, error)
	SoftDeleteBotUser(user *models.BotUser) error
}

type gormAdminRepository struct {
	db *gorm.DB
}

func NewAdminRepository(db *gorm.DB) AdminRepository {
	return &gormAdminRepository{db: db}
}

func (r *gormAdminRepository) GetActiveBotUsers() ([]models.BotUser, error) {
	var botUsers []models.BotUser
	if err := r.db.Where("is_deleted = ?", false).Find(&botUsers).Error; err != nil {
		return nil, err
	}
	return botUsers, nil
}

func (r *gormAdminRepository) GetBotUserByID(id uint) (*models.BotUser, error) {
	var botUser models.BotUser
	if err := r.db.First(&botUser, id).Error; err != nil {
		return nil, err
	}
	return &botUser, nil
}

func (r *gormAdminRepository) SoftDeleteBotUser(user *models.BotUser) error {
	user.IsDeleted = true
	return r.db.Save(user).Error
}
