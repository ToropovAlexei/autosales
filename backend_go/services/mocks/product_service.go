package mocks

import (
	"frbktg/backend_go/models"

	"github.com/stretchr/testify/mock"
)

type MockProductService struct{ mock.Mock }

func (m *MockProductService) GetProducts(categoryIDs []string) ([]models.ProductResponse, error) {
	args := m.Called(categoryIDs)
	return args.Get(0).([]models.ProductResponse), args.Error(1)
}
func (m *MockProductService) GetProduct(id uint) (*models.ProductResponse, error) {
	args := m.Called(id)
	return args.Get(0).(*models.ProductResponse), args.Error(1)
}
func (m *MockProductService) CreateProduct(name string, categoryID uint, price float64, initialStock int, productType string, subscriptionPeriodDays int) (*models.ProductResponse, error) {
	args := m.Called(name, categoryID, price, initialStock, productType, subscriptionPeriodDays)
	return args.Get(0).(*models.ProductResponse), args.Error(1)
}
func (m *MockProductService) UpdateProduct(id uint, data models.Product) (*models.ProductResponse, error) {
	args := m.Called(id, data)
	return args.Get(0).(*models.ProductResponse), args.Error(1)
}
func (m *MockProductService) DeleteProduct(id uint) error {
	return m.Called(id).Error(0)
}
func (m *MockProductService) CreateStockMovement(productID uint, movementType models.StockMovementType, quantity int, description string, orderID *uint) (*models.StockMovement, error) {
	args := m.Called(productID, movementType, quantity, description, orderID)
	return args.Get(0).(*models.StockMovement), args.Error(1)
}
