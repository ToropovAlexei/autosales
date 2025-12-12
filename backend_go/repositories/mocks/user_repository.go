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
func (m *MockUserRepository) FindByLogin(login string) (*models.User, error) { args := m.Called(login); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.User), args.Error(1) }
func (m *MockUserRepository) FindByTelegramID(telegramID int64) (*models.User, error) { args := m.Called(telegramID); if args.Get(0) == nil { return nil, args.Error(1) }; return args.Get(0).(*models.User), args.Error(1) }
func (m *MockUserRepository) Create(user *models.User) error { return m.Called(user).Error(0) }
func (m *MockUserRepository) Update(user *models.User) error { return m.Called(user).Error(0) }
func (m *MockUserRepository) GetUsers(page models.Page, filters []models.Filter) (*models.PaginatedResult[models.User], error) { return nil, nil }
func (m *MockUserRepository) SetUserRole(userID, roleID uint) error { return m.Called(userID, roleID).Error(0) }
func (m *MockUserRepository) SetTelegramID(userID uint, telegramID int64) error { return m.Called(userID, telegramID).Error(0) }
