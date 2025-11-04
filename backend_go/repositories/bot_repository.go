package repositories

import (
	"errors"
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type BotRepository interface {
	FindByName(name string) (*models.Bot, error)
	FindByID(id uint) (*models.Bot, error)
	Create(bot *models.Bot) error
	CountByOwnerID(ownerID uint) (int64, error)
	FindByToken(token string) (*models.Bot, error)
	GetAll(botType string) ([]models.Bot, error)
	WithTx(tx *gorm.DB) BotRepository
}

type gormBotRepository struct {
	db *gorm.DB
}

func NewBotRepository(db *gorm.DB) BotRepository {
	return &gormBotRepository{db: db}
}

func (r *gormBotRepository) WithTx(tx *gorm.DB) BotRepository {
	return &gormBotRepository{db: tx}
}

func (r *gormBotRepository) FindByName(name string) (*models.Bot, error) {
	var bot models.Bot
	err := r.db.Where("username = ?", name).First(&bot).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}
	return &bot, nil
}

func (r *gormBotRepository) FindByToken(token string) (*models.Bot, error) {
	var bot models.Bot
	err := r.db.Where("token = ?", token).First(&bot).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}
	return &bot, nil
}

func (r *gormBotRepository) FindByID(id uint) (*models.Bot, error) {
	var bot models.Bot
	if err := r.db.First(&bot, id).Error; err != nil {
		return nil, err
	}
	return &bot, nil
}

func (r *gormBotRepository) Create(bot *models.Bot) error {
	return r.db.Create(bot).Error
}

func (r *gormBotRepository) CountByOwnerID(ownerID uint) (int64, error) {
	var count int64
	err := r.db.Model(&models.Bot{}).Where("owner_id = ?", ownerID).Count(&count).Error
	return count, err
}

func (r *gormBotRepository) GetAll(botType string) ([]models.Bot, error) {
	var bots []models.Bot
	db := r.db
	if botType != "" {
		db = db.Where("type = ?", botType)
	}
	if err := db.Find(&bots).Error; err != nil {
		return nil, err
	}
	return bots, nil
}
