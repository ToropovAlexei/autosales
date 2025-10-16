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
	GetUsers(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.User], error)
	Create(user *models.User) error
	SetUserRole(userID, roleID uint) error
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
	if err := r.db.Preload("Roles").Where("email = ?", email).First(&user).Error; err != nil {
		return nil, err
	}
	return &user, nil
}

func (r *gormUserRepository) GetUsers(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.User], error) {
	db := r.db.Preload("Roles")
	db = ApplyFilters[models.User](db, filters)
	return ApplyPagination[models.User](db, page)
}

func (r *gormUserRepository) Create(user *models.User) error {
	return r.db.Create(user).Error
}

func (r *gormUserRepository) SetUserRole(userID, roleID uint) error {
	userRole := models.UserRole{UserID: userID, RoleID: roleID}
	return r.db.FirstOrCreate(&userRole, userRole).Error
}
