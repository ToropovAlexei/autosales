package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type UserRepository interface {
	WithTx(tx *gorm.DB) UserRepository
	UpdateReferralSettings(user *models.User, enabled bool, percentage float64) error
	FindByID(id uint) (*models.User, error)
	FindByEmail(email string) (*models.User, error)
}

type gormUserRepository struct {
	db *gorm.DB
}

func NewUserRepository(db *gorm.DB) UserRepository {
	return &gormUserRepository{db: db}
}

func (r *gormUserRepository) WithTx(tx *gorm.DB) UserRepository {
	return &gormUserRepository{db: tx}
}

func (r *gormUserRepository) UpdateReferralSettings(user *models.User, enabled bool, percentage float64) error {
	return r.db.Model(user).Updates(models.User{
		ReferralProgramEnabled: enabled,
		ReferralPercentage:     percentage,
	}).Error
}

func (r *gormUserRepository) FindByID(id uint) (*models.User, error) {
	var user models.User
	if err := r.db.First(&user, id).Error; err != nil {
		return nil, err
	}
	return &user, nil
}

func (r *gormUserRepository) FindByEmail(email string) (*models.User, error) {
	var user models.User
	if err := r.db.Where("email = ?", email).First(&user).Error; err != nil {
		return nil, err
	}
	return &user, nil
}
