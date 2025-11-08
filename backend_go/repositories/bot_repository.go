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
	Delete(botID uint) error
	Update(bot *models.Bot, column string, value interface{}) error
	SetPrimary(bot *models.Bot) error
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
	bots := make([]models.Bot, 0)
	db := r.db
	if botType != "" {
		db = db.Where("type = ?", botType)
	}
	if err := db.Find(&bots).Error; err != nil {
		return nil, err
	}
	return bots, nil
}

func (r *gormBotRepository) Delete(botID uint) error {
	return r.db.Delete(&models.Bot{}, botID).Error
}

func (r *gormBotRepository) Update(bot *models.Bot, column string, value interface{}) error {
	return r.db.Model(bot).Update(column, value).Error
}

func (r *gormBotRepository) SetPrimary(bot *models.Bot) error {
	return r.db.Transaction(func(tx *gorm.DB) error {
		if err := tx.Model(&models.Bot{}).Where("is_primary = ?", true).Update("is_primary", false).Error; err != nil {
			return err
		}
		return tx.Model(bot).Update("is_primary", true).Error
	})
}
