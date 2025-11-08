package repositories

import (
	"frbktg/backend_go/models"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type ImageRepository interface {
	WithTx(tx *gorm.DB) ImageRepository
	Create(image *models.Image) error
	FindByHash(hash string) (*models.Image, error)
	FindByID(id uuid.UUID) (*models.Image, error)
	ListByFolder(folder string) ([]models.Image, error)
	Delete(id uuid.UUID) error
}

type gormImageRepository struct {
	db *gorm.DB
}

func NewImageRepository(db *gorm.DB) ImageRepository {
	return &gormImageRepository{db: db}
}

func (r *gormImageRepository) Delete(id uuid.UUID) error {
	return r.db.Delete(&models.Image{}, "id = ?", id).Error
}

func (r *gormImageRepository) WithTx(tx *gorm.DB) ImageRepository {
	return &gormImageRepository{db: tx}
}

func (r *gormImageRepository) Create(image *models.Image) error {
	return r.db.Create(image).Error
}

func (r *gormImageRepository) FindByHash(hash string) (*models.Image, error) {
	var image models.Image
	if err := r.db.Where("hash = ?", hash).First(&image).Error; err != nil {
		return nil, err
	}
	return &image, nil
}

func (r *gormImageRepository) FindByID(id uuid.UUID) (*models.Image, error) {
	var image models.Image
	if err := r.db.Where("id = ?", id).First(&image).Error; err != nil {
		return nil, err
	}
	return &image, nil
}

func (r *gormImageRepository) ListByFolder(folder string) ([]models.Image, error) {
	images := make([]models.Image, 0)
	query := r.db.Order("id asc")
	if folder != "" {
		query = query.Where("folder = ?", folder)
	} else {
		query = query.Where("folder = '' OR folder IS NULL")
	}
	if err := query.Find(&images).Error; err != nil {
		return nil, err
	}
	return images, nil
}
