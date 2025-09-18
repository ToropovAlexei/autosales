package services

import (
	"errors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

// MockProductRepository is a mock type for the ProductRepository interface
type MockProductRepository struct {
	mock.Mock
}

func (m *MockProductRepository) WithTx(tx *gorm.DB) repositories.ProductRepository { // Corrected type
	m.Called(tx)
	return m
}

func (m *MockProductRepository) GetProducts(categoryIDs []string) ([]models.Product, error) {
	args := m.Called(categoryIDs)
	return args.Get(0).([]models.Product), args.Error(1)
}

func (m *MockProductRepository) GetProductByID(id uint) (*models.Product, error) {
	args := m.Called(id)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Product), args.Error(1)
}

func (m *MockProductRepository) CreateProduct(product *models.Product) error {
	args := m.Called(product)
	return args.Error(0)
}

func (m *MockProductRepository) UpdateProduct(product *models.Product, data models.Product) error {
	args := m.Called(product, data)
	return args.Error(0)
}

func (m *MockProductRepository) DeleteProduct(product *models.Product) error {
	args := m.Called(product)
	return args.Error(0)
}

func (m *MockProductRepository) GetStockForProduct(productID uint) (int, error) {
	args := m.Called(productID)
	return args.Int(0), args.Error(1)
}

func (m *MockProductRepository) CreateStockMovement(movement *models.StockMovement) error {
	args := m.Called(movement)
	return args.Error(0)
}

func (m *MockProductRepository) FindCategoryByID(id uint) (*models.Category, error) {
	args := m.Called(id)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Category), args.Error(1)
}

func TestProductService_GetProduct(t *testing.T) {
	// Arrange
	mockRepo := new(MockProductRepository)
	productService := NewProductService(mockRepo)

	expectedProduct := &models.Product{ID: 1, Name: "Test Product", Price: 100, CategoryID: 1}
	expectedStock := 10

	mockRepo.On("GetProductByID", uint(1)).Return(expectedProduct, nil)
	mockRepo.On("GetStockForProduct", uint(1)).Return(expectedStock, nil)

	// Act
	productResponse, err := productService.GetProduct(1)

	// Assert
	assert.NoError(t, err)
	assert.NotNil(t, productResponse)
	assert.Equal(t, expectedProduct.ID, productResponse.ID)
	assert.Equal(t, expectedProduct.Name, productResponse.Name)
	assert.Equal(t, expectedStock, productResponse.Stock)
	mockRepo.AssertExpectations(t)
}

func TestProductService_GetProduct_NotFound(t *testing.T) {
	// Arrange
	mockRepo := new(MockProductRepository)
	productService := NewProductService(mockRepo)

	mockRepo.On("GetProductByID", uint(1)).Return(nil, errors.New("not found"))

	// Act
	productResponse, err := productService.GetProduct(1)

	// Assert
	assert.Error(t, err)
	assert.Nil(t, productResponse)
	assert.Equal(t, "not found", err.Error())
	mockRepo.AssertExpectations(t)
}

func TestProductService_CreateProduct(t *testing.T) {
	// Arrange
	mockRepo := new(MockProductRepository)
	productService := NewProductService(mockRepo)

	category := &models.Category{ID: 1, Name: "Test"}
	initialStock := 50

	// We need to use mock.AnythingOfType because the pointer to the product/movement will be different.
	mockRepo.On("FindCategoryByID", uint(1)).Return(category, nil)
	mockRepo.On("CreateProduct", mock.AnythingOfType("*models.Product")).Return(nil)
	mockRepo.On("CreateStockMovement", mock.AnythingOfType("*models.StockMovement")).Return(nil)

	// Act
	productResponse, err := productService.CreateProduct("New Product", 1, 150.0, initialStock)

	// Assert
	assert.NoError(t, err)
	assert.NotNil(t, productResponse)
	assert.Equal(t, "New Product", productResponse.Name)
	assert.Equal(t, 150.0, productResponse.Price)
	assert.Equal(t, initialStock, productResponse.Stock)
	mockRepo.AssertExpectations(t)
}

func TestProductService_CreateProduct_CategoryNotFound(t *testing.T) {
	// Arrange
	mockRepo := new(MockProductRepository)
	productService := NewProductService(mockRepo)

	mockRepo.On("FindCategoryByID", uint(99)).Return(nil, errors.New("category not found"))

	// Act
	productResponse, err := productService.CreateProduct("New Product", 99, 150.0, 50)

	// Assert
	assert.Error(t, err)
	assert.Nil(t, productResponse)
	assert.Equal(t, "category not found", err.Error())
	mockRepo.AssertExpectations(t)
}
