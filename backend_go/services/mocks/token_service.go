package mocks

import (
	"frbktg/backend_go/models"

	"github.com/golang-jwt/jwt/v5"
	"github.com/stretchr/testify/mock"
)

type MockTokenService struct{ mock.Mock }

func (m *MockTokenService) GenerateToken(user *models.User, secretKey string, expireMinutes int) (string, error) { 
	args := m.Called(user, secretKey, expireMinutes)
	return args.String(0), args.Error(1) 
}
func (m *MockTokenService) ValidateToken(tokenString string, secretKey string) (*jwt.Token, error) { 
	args := m.Called(tokenString, secretKey)
	return args.Get(0).(*jwt.Token), args.Error(1) 
}
