package mocks

import "github.com/stretchr/testify/mock"

type MockAuthService struct{ mock.Mock }

func (m *MockAuthService) Login(email, password string) (string, error) {
	args := m.Called(email, password)
	return args.String(0), args.Error(1)
}
