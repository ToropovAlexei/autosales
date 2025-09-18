package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type CategoryRepository interface {
	GetAll() ([]models.Category, error)
	GetByID(id uint) (*models.Category, error)
	Create(category *models.Category) error
	Update(category *models.Category, data models.Category) error
	Delete(category *models.Category) error
}

type gormCategoryRepository struct {
	db *gorm.DB
}

func NewCategoryRepository(db *gorm.DB) CategoryRepository {
	return &gormCategoryRepository{db: db}
}

func (r *gormCategoryRepository) GetAll() ([]models.Category, error) {
	var categories []models.Category
	if err := r.db.Find(&categories).Error; err != nil {
		return nil, err
	}
	return categories, nil
}

func (r *gormCategoryRepository) GetByID(id uint) (*models.Category, error) {
	var category models.Category
	if err := r.db.First(&category, id).Error; err != nil {
		return nil, err
	}
	return &category, nil
}

func (r *gormCategoryRepository) Create(category *models.Category) error {
	return r.db.Create(category).Error
}

func (r *gormCategoryRepository) Update(category *models.Category, data models.Category) error {
	return r.db.Model(category).Updates(data).Error
}

func (r *gormCategoryRepository) Delete(category *models.Category) error {
	return r.db.Delete(category).Error
}
