package mocks

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

type MockUserRepository struct{ mock.Mock }

func (m *MockUserRepository) WithTx(tx *gorm.DB) repositories.UserRepository { m.Called(tx); return m }
func (m *MockUserRepository) UpdateReferralSettings(user *models.User, enabled bool, percentage float64) error { return m.Called(user, enabled, percentage).Error(0) }
func (m *MockUserRepository) FindSellerSettings() (*models.User, error) { args := m.Called(); return args.Get(0).(*models.User), args.Error(1) }
func (m *MockUserRepository) FindByID(id uint) (*models.User, error) { args := m.Called(id); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.User), args.Error(1) }
func (m *MockUserRepository) FindByEmail(email string) (*models.User, error) { args := m.Called(email); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.User), args.Error(1) }
