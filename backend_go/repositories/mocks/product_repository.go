package mocks

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

type MockProductRepository struct{ mock.Mock }

func (m *MockProductRepository) WithTx(tx *gorm.DB) repositories.ProductRepository {
	m.Called(tx)
	return m
}
func (m *MockProductRepository) GetProducts(filters []models.Filter) ([]models.Product, error) {
	return nil, nil
}
func (m *MockProductRepository) GetProductByID(id uint) (*models.Product, error) {
	args := m.Called(id)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Product), args.Error(1)
}
func (m *MockProductRepository) FindByName(name string) (*models.Product, error) {
	args := m.Called(name)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Product), args.Error(1)
}
func (m *MockProductRepository) CreateProduct(product *models.Product) error {
	return m.Called(product).Error(0)
}
func (m *MockProductRepository) UpdateProduct(product *models.Product, data map[string]interface{}) error {
	return m.Called(product, data).Error(0)
}
func (m *MockProductRepository) DeleteProduct(product *models.Product) error {
	return m.Called(product).Error(0)
}
func (m *MockProductRepository) GetStockForProduct(productID uint) (int, error) {
	args := m.Called(productID)
	return args.Int(0), args.Error(1)
}
func (m *MockProductRepository) CreateStockMovement(movement *models.StockMovement) error {
	return m.Called(movement).Error(0)
}
func (m *MockProductRepository) FindCategoryByID(id uint) (*models.Category, error) {
	args := m.Called(id)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Category), args.Error(1)
}
func (m *MockProductRepository) GetExternalProducts() ([]models.Product, error) {
	args := m.Called()
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]models.Product), args.Error(1)
}