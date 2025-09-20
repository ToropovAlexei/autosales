package services

import (
	"errors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories/mocks"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestCategoryService_GetAll(t *testing.T) {
	// Arrange
	mockRepo := new(mocks.MockCategoryRepository)
	categoryService := NewCategoryService(mockRepo)

	expectedCategories := []models.Category{{ID: 1, Name: "Cat1"}, {ID: 2, Name: "Cat2"}}
	mockRepo.On("GetAll").Return(expectedCategories, nil)

	// Act
	categories, err := categoryService.GetAll()

	// Assert
	assert.NoError(t, err)
	assert.Len(t, categories, 2)
	assert.Equal(t, "Cat1", categories[0].Name)
	mockRepo.AssertExpectations(t)
}

func TestCategoryService_Create(t *testing.T) {
	// Arrange
	mockRepo := new(mocks.MockCategoryRepository)
	categoryService := NewCategoryService(mockRepo)

	mockRepo.On("Create", mock.AnythingOfType("*models.Category")).Return(nil)

	// Act
	category, err := categoryService.Create("New Category")

	// Assert
	assert.NoError(t, err)
	assert.NotNil(t, category)
	assert.Equal(t, "New Category", category.Name)
	mockRepo.AssertExpectations(t)
}

func TestCategoryService_Update_NotFound(t *testing.T) {
	// Arrange
	mockRepo := new(mocks.MockCategoryRepository)
	categoryService := NewCategoryService(mockRepo)

	mockRepo.On("GetByID", uint(99)).Return(nil, errors.New("not found"))

	// Act
	category, err := categoryService.Update(99, "Updated Name")

	// Assert
	assert.Error(t, err)
	assert.Nil(t, category)
	mockRepo.AssertExpectations(t)
}
