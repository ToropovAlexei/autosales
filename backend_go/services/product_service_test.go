package services

import (
	"errors"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories/mocks"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

func TestProductService_GetProduct(t *testing.T) {
	// Arrange
	mockRepo := new(mocks.MockProductRepository)
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
	mockRepo := new(mocks.MockProductRepository)
	productService := NewProductService(mockRepo)

	mockRepo.On("GetProductByID", uint(1)).Return(nil, gorm.ErrRecordNotFound)

	// Act
	productResponse, err := productService.GetProduct(1)

	// Assert
	assert.Error(t, err)
	assert.Nil(t, productResponse)
	assert.IsType(t, &apperrors.ErrNotFound{}, err)
	mockRepo.AssertExpectations(t)
}

func TestProductService_CreateProduct(t *testing.T) {
	// Arrange
	mockRepo := new(mocks.MockProductRepository)
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
	mockRepo := new(mocks.MockProductRepository)
	productService := NewProductService(mockRepo)

	mockRepo.On("FindCategoryByID", uint(99)).Return(nil, errors.New("category not found"))

	// Act
	productResponse, err := productService.CreateProduct("New Product", 99, 150.0, 50)

	// Assert
	assert.Error(t, err)
	assert.Nil(t, productResponse)
	assert.IsType(t, &apperrors.ErrNotFound{}, err)
	mockRepo.AssertExpectations(t)
}
