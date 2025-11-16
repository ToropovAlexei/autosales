package mocks

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

type MockOrderRepository struct{ mock.Mock }

func (m *MockOrderRepository) WithTx(tx *gorm.DB) repositories.OrderRepository {
	m.Called(tx)
	return m
}
func (m *MockOrderRepository) CreateOrder(order *models.Order) error { return m.Called(order).Error(0) }
func (m *MockOrderRepository) GetOrderForUpdate(id uint) (*models.Order, error) {
	args := m.Called(id)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Order), args.Error(1)
}
func (m *MockOrderRepository) UpdateOrder(order *models.Order) error { return m.Called(order).Error(0) }
func (m *MockOrderRepository) GetOrders(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.OrderResponse], error) {
	args := m.Called(page, filters)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.PaginatedResult[models.OrderResponse]), args.Error(1)
}
func (m *MockOrderRepository) FindOrdersByBotUserID(botUserID uint) ([]models.Order, error) {
	args := m.Called(botUserID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]models.Order), args.Error(1)
}
func (m *MockOrderRepository) GetOrderByID(id uint) (*models.Order, error) {
	args := m.Called(id)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Order), args.Error(1)
}
