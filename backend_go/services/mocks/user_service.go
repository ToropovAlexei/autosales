package mocks

import (
	"frbktg/backend_go/models"

	"github.com/stretchr/testify/mock"
)

type MockUserService struct{ mock.Mock }

func (m *MockUserService) GetMe(user models.User) *models.UserResponse {
	args := m.Called(user)
	return args.Get(0).(*models.UserResponse)
}
func (m *MockUserService) UpdateReferralSettings(user *models.User, enabled bool, percentage float64) error {
	return m.Called(user, enabled, percentage).Error(0)
}
func (m *MockUserService) RegisterBotUser(telegramID int64) (*models.BotUser, float64, bool, bool, error) {
	args := m.Called(telegramID)
	return args.Get(0).(*models.BotUser), args.Get(1).(float64), args.Bool(2), args.Bool(3), args.Error(4)
}
func (m *MockUserService) GetBotUser(id uint) (*models.BotUser, float64, error) {
	args := m.Called(id)
	return args.Get(0).(*models.BotUser), args.Get(1).(float64), args.Error(2)
}
func (m *MockUserService) GetUserBalance(telegramID int64) (float64, error) {
	args := m.Called(telegramID)
	return args.Get(0).(float64), args.Error(1)
}
func (m *MockUserService) GetUserTransactions(telegramID int64) ([]models.Transaction, error) {
	args := m.Called(telegramID)
	return args.Get(0).([]models.Transaction), args.Error(1)
}
func (m *MockUserService) UpdateUserCaptchaStatus(id uint, hasPassed bool) error {
	return m.Called(id, hasPassed).Error(0)
}
func (m *MockUserService) GetSellerSettings() (*models.User, error) {
	args := m.Called()
	return args.Get(0).(*models.User), args.Error(1)
}
