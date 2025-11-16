package mocks

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

type MockBotUserRepository struct{ mock.Mock }

func (m *MockBotUserRepository) WithTx(tx *gorm.DB) repositories.BotUserRepository {
	m.Called(tx)
	return m
}
func (m *MockBotUserRepository) FindByTelegramID(telegramID int64) (*models.BotUser, error) {
	args := m.Called(telegramID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.BotUser), args.Error(1)
}
func (m *MockBotUserRepository) FindByID(id uint) (*models.BotUser, error) {
	args := m.Called(id)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.BotUser), args.Error(1)
}
func (m *MockBotUserRepository) Create(user *models.BotUser) error { return m.Called(user).Error(0) }
func (m *MockBotUserRepository) Update(user *models.BotUser) error { return m.Called(user).Error(0) }
func (m *MockBotUserRepository) UpdateCaptchaStatus(user *models.BotUser, hasPassed bool) error {
	return m.Called(user, hasPassed).Error(0)
}
func (m *MockBotUserRepository) GetUserBalance(userID uint) (float64, error) {
	args := m.Called(userID)
	return args.Get(0).(float64), args.Error(1)
}
func (m *MockBotUserRepository) GetUserTransactions(userID uint) ([]models.Transaction, error) {
	args := m.Called(userID)
	return args.Get(0).([]models.Transaction), args.Error(1)
}
func (m *MockBotUserRepository) UpdateBalance(userID uint, amount float64) error {
	return m.Called(userID, amount).Error(0)
}