package mocks

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

type MockAdminRepository struct{ mock.Mock }

func (m *MockAdminRepository) WithTx(tx *gorm.DB) repositories.AdminRepository {
	m.Called(tx)
	return m
}
func (m *MockAdminRepository) GetActiveBotUsers(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.BotUser], error) {
	args := m.Called(page, filters)
	return args.Get(0).(*models.PaginatedResult[models.BotUser]), args.Error(1)
}
func (m *MockAdminRepository) GetBotUserByID(id uint) (*models.BotUser, error) {
	args := m.Called(id)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.BotUser), args.Error(1)
}