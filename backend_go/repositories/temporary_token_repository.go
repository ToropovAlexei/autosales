package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type TemporaryTokenRepository interface {
	Create(token *models.TemporaryToken) error
	Find(token string) (*models.TemporaryToken, error)
	Delete(token string) error
}

type temporaryTokenRepository struct {
	db *gorm.DB
}

func NewTemporaryTokenRepository(db *gorm.DB) TemporaryTokenRepository {
	return &temporaryTokenRepository{db: db}
}

func (r *temporaryTokenRepository) Create(token *models.TemporaryToken) error {
	return r.db.Create(token).Error
}

func (r *temporaryTokenRepository) Find(token string) (*models.TemporaryToken, error) {
	var tempToken models.TemporaryToken
	if err := r.db.First(&tempToken, "token = ?", token).Error; err != nil {
		return nil, err
	}
	return &tempToken, nil
}

func (r *temporaryTokenRepository) Delete(token string) error {
	return r.db.Where("token = ?", token).Delete(&models.TemporaryToken{}).Error
}
