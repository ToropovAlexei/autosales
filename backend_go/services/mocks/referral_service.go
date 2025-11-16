package mocks

import (
	"frbktg/backend_go/models"

	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

type MockReferralService struct{ mock.Mock }

func (m *MockReferralService) ProcessReferral(tx *gorm.DB, token *string, order models.Order, amount float64) error {
	return m.Called(tx, token, order, amount).Error(0)
}
func (m *MockReferralService) CreateReferralBot(ownerTelegramID int64, sellerID uint, botToken string) (*models.Bot, error) {
	args := m.Called(ownerTelegramID, sellerID, botToken)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Bot), args.Error(1)
}
func (m *MockReferralService) GetAllReferralBots() ([]models.BotResponse, error) {
	args := m.Called()
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]models.BotResponse), args.Error(1)
}
func (m *MockReferralService) GetAdminInfoForSeller(sellerID uint) ([]models.BotResponse, error) {
	args := m.Called(sellerID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]models.BotResponse), args.Error(1)
}
func (m *MockReferralService) ToggleReferralBotStatus(botID uint, sellerID uint) (*models.Bot, error) {
	args := m.Called(botID, sellerID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Bot), args.Error(1)
}
