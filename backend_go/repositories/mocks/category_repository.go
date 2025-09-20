package mocks

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

type MockCategoryRepository struct{ mock.Mock }

func (m *MockCategoryRepository) WithTx(tx *gorm.DB) repositories.CategoryRepository { m.Called(tx); return m }
func (m *MockCategoryRepository) GetAll() ([]models.Category, error) { args := m.Called(); return args.Get(0).([]models.Category), args.Error(1) }
func (m *MockCategoryRepository) GetByID(id uint) (*models.Category, error) { args := m.Called(id); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.Category), args.Error(1) }
func (m *MockCategoryRepository) Create(category *models.Category) error { return m.Called(category).Error(0) }
func (m *MockCategoryRepository) Update(category *models.Category, data models.Category) error { return m.Called(category, data).Error(0) }
func (m *MockCategoryRepository) Delete(category *models.Category) error { return m.Called(category).Error(0) }
