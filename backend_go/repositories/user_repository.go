package repositories

import (
	"frbktg/backend_go/models"
	"gorm.io/gorm"
)

type UserRepository interface {
	UpdateReferralSettings(user *models.User, enabled bool, percentage float64) error
	FindSellerSettings() (*models.User, error)
}

type gormUserRepository struct {
	db *gorm.DB
}

func NewUserRepository(db *gorm.DB) UserRepository {
	return &gormUserRepository{db: db}
}

func (r *gormUserRepository) UpdateReferralSettings(user *models.User, enabled bool, percentage float64) error {
	return r.db.Model(user).Updates(models.User{
		ReferralProgramEnabled: enabled,
		ReferralPercentage:     percentage,
	}).Error
}

func (r *gormUserRepository) FindSellerSettings() (*models.User, error) {
	var seller models.User
	if err := r.db.Where("role = ?", models.Admin).First(&seller).Error; err != nil {
		if err = r.db.First(&seller).Error; err != nil {
			return nil, err
		}
	}
	return &seller, nil
}
