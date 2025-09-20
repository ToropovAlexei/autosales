package mocks

import (
	"frbktg/backend_go/repositories"
	"time"

	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

type MockDashboardRepository struct{ mock.Mock }

func (m *MockDashboardRepository) WithTx(tx *gorm.DB) repositories.DashboardRepository { m.Called(tx); return m }
func (m *MockDashboardRepository) CountTotalUsers() (int64, error) { args := m.Called(); return args.Get(0).(int64), args.Error(1) }
func (m *MockDashboardRepository) CountUsersWithPurchases() (int64, error) { args := m.Called(); return args.Get(0).(int64), args.Error(1) }
func (m *MockDashboardRepository) CountAvailableProducts() (int64, error) { args := m.Called(); return args.Get(0).(int64), args.Error(1) }
func (m *MockDashboardRepository) GetSalesCountForPeriod(start, end time.Time) (int64, error) { args := m.Called(start, end); return args.Get(0).(int64), args.Error(1) }
func (m *MockDashboardRepository) GetTotalRevenueForPeriod(start, end time.Time) (float64, error) { args := m.Called(start, end); return args.Get(0).(float64), args.Error(1) }
