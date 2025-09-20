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
func (m *MockReferralService) CreateReferralBot(ownerTelegramID int64, sellerID uint, botToken string) (*models.ReferralBot, error) { 
	return nil, nil 
}
func (m *MockReferralService) GetAllReferralBots() ([]models.ReferralBotResponse, error) { 
	return nil, nil 
}
func (m *MockReferralService) GetAdminInfoForSeller(sellerID uint) ([]models.ReferralBotAdminInfo, error) { 
	return nil, nil 
}
func (m *MockReferralService) ToggleReferralBotStatus(botID uint, sellerID uint) (*models.ReferralBot, error) { 
	return nil, nil 
}
