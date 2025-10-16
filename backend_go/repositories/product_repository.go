package repositories

import (
	"errors"
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type ProductRepository interface {
	WithTx(tx *gorm.DB) ProductRepository
	GetProducts(filters []models.Filter) ([]models.Product, error)
	GetProductByID(id uint) (*models.Product, error)
	FindByName(name string) (*models.Product, error)
	CreateProduct(product *models.Product) error
	UpdateProduct(product *models.Product, data map[string]interface{}) error
	DeleteProduct(product *models.Product) error
	GetStockForProduct(productID uint) (int, error)
	CreateStockMovement(movement *models.StockMovement) error
	FindCategoryByID(id uint) (*models.Category, error)
}

type gormProductRepository struct {
	db *gorm.DB
}

func NewProductRepository(db *gorm.DB) ProductRepository {
	return &gormProductRepository{db: db}
}

func (r *gormProductRepository) WithTx(tx *gorm.DB) ProductRepository {
	return &gormProductRepository{db: tx}
}

func (r *gormProductRepository) GetProducts(filters []models.Filter) ([]models.Product, error) {
	var products []models.Product
	db := r.db.Model(&models.Product{}).Where("visible = ?", true)
	db = ApplyFilters[models.Product](db, filters)
	if err := db.Find(&products).Error; err != nil {
		return nil, err
	}
	return products, nil
}

func (r *gormProductRepository) GetProductByID(id uint) (*models.Product, error) {
	var product models.Product
	if err := r.db.First(&product, id).Error; err != nil {
		return nil, err
	}
	return &product, nil
}

func (r *gormProductRepository) FindByName(name string) (*models.Product, error) {
	var product models.Product
	if err := r.db.Where("name = ?", name).First(&product).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}
	return &product, nil
}

func (r *gormProductRepository) CreateProduct(product *models.Product) error {
	return r.db.Create(product).Error
}

func (r *gormProductRepository) UpdateProduct(product *models.Product, data map[string]interface{}) error {
	return r.db.Model(product).Updates(data).Error
}

func (r *gormProductRepository) DeleteProduct(product *models.Product) error {
	return r.db.Delete(product).Error
}

func (r *gormProductRepository) GetStockForProduct(productID uint) (int, error) {
	var stock int64
	if err := r.db.Model(&models.StockMovement{}).Where("product_id = ?", productID).Select("sum(quantity)").
		Row().Scan(&stock); err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		return 0, err
	}
	return int(stock), nil
}

func (r *gormProductRepository) CreateStockMovement(movement *models.StockMovement) error {
	return r.db.Create(movement).Error
}

func (r *gormProductRepository) FindCategoryByID(id uint) (*models.Category, error) {
	var category models.Category
	if err := r.db.First(&category, id).Error; err != nil {
		return nil, err
	}
	return &category, nil
}
