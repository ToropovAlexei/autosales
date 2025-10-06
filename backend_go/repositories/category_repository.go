package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type CategoryRepository interface {
	WithTx(tx *gorm.DB) CategoryRepository
	GetAll() ([]models.Category, error)
	GetByID(id uint) (*models.Category, error)
	FindByName(name string) (*models.Category, error)
	FindByNameAndParent(name string, parentID *uint) (*models.Category, error)
	Create(category *models.Category) error
	Update(category *models.Category, data models.Category) error
	Delete(category *models.Category) error
	FindOrCreateByPath(path []string) (*models.Category, error)
}

type gormCategoryRepository struct {
	db *gorm.DB
}

func NewCategoryRepository(db *gorm.DB) CategoryRepository {
	return &gormCategoryRepository{db: db}
}

func (r *gormCategoryRepository) WithTx(tx *gorm.DB) CategoryRepository {
	return &gormCategoryRepository{db: tx}
}

func (r *gormCategoryRepository) GetAll() ([]models.Category, error) {
	var categories []models.Category
	if err := r.db.Order("id asc").Find(&categories).Error; err != nil {
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

func (r *gormCategoryRepository) FindByName(name string) (*models.Category, error) {
	var category models.Category
	if err := r.db.Where("name = ?", name).First(&category).Error; err != nil {
		return nil, err
	}
	return &category, nil
}

func (r *gormCategoryRepository) FindByNameAndParent(name string, parentID *uint) (*models.Category, error) {
	var category models.Category
	query := r.db.Where("name = ?", name)
	if parentID == nil {
		query = query.Where("parent_id IS NULL")
	} else {
		query = query.Where("parent_id = ?", *parentID)
	}
	if err := query.First(&category).Error; err != nil {
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

func (r *gormCategoryRepository) FindOrCreateByPath(path []string) (*models.Category, error) {
	var parentID *uint
	var currentCategory *models.Category

	for _, name := range path {
		var category models.Category
		query := models.Category{Name: name, ParentID: parentID}

		if err := r.db.Where(query).FirstOrCreate(&category).Error; err != nil {
			return nil, err
		}

		currentCategory = &category
		parentID = &currentCategory.ID
	}

	return currentCategory, nil
}
