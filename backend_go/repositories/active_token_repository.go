package repositories

import (
	"frbktg/backend_go/models"
	"time"

	"gorm.io/gorm"
)

type ActiveTokenRepository interface {
	Create(jti string, userID uint, expiresAt time.Time) error
	Exists(jti string) (bool, error)
	Delete(jti string) error
}

type activeTokenRepository struct {
	db *gorm.DB
}

func NewActiveTokenRepository(db *gorm.DB) ActiveTokenRepository {
	return &activeTokenRepository{db: db}
}

func (r *activeTokenRepository) Create(jti string, userID uint, expiresAt time.Time) error {
	token := models.ActiveToken{JTI: jti, UserID: userID, ExpiresAt: expiresAt}
	return r.db.Create(&token).Error
}

func (r *activeTokenRepository) Exists(jti string) (bool, error) {
	var count int64
	if err := r.db.Model(&models.ActiveToken{}).Where("jti = ?", jti).Count(&count).Error; err != nil {
		return false, err
	}
	return count > 0, nil
}

func (r *activeTokenRepository) Delete(jti string) error {
	return r.db.Where("jti = ?", jti).Delete(&models.ActiveToken{}).Error
}
