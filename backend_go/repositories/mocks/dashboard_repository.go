package mocks

import (
	"frbktg/backend_go/models"
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
func (m *MockDashboardRepository) CountProductsSoldForPeriod(start, end time.Time) (int64, error) { args := m.Called(start, end); return args.Get(0).(int64), args.Error(1) }
func (m *MockDashboardRepository) CountTotalUsersForPeriod(start, end time.Time) (int64, error) { args := m.Called(start, end); return args.Get(0).(int64), args.Error(1) }
func (m *MockDashboardRepository) CountUsersWithPurchasesForPeriod(start, end time.Time) (int64, error) { args := m.Called(start, end); return args.Get(0).(int64), args.Error(1) }
func (m *MockDashboardRepository) GetRevenueOverTime(start, end time.Time) ([]models.TimeSeriesData, error) { args := m.Called(start, end); return args.Get(0).([]models.TimeSeriesData), args.Error(1) }
func (m *MockDashboardRepository) GetSalesByCategory() ([]models.CategorySales, error) { args := m.Called(); return args.Get(0).([]models.CategorySales), args.Error(1) }
func (m *MockDashboardRepository) GetSalesOverTime(start, end time.Time) ([]models.TimeSeriesData, error) { args := m.Called(start, end); return args.Get(0).([]models.TimeSeriesData), args.Error(1) }
func (m *MockDashboardRepository) GetTopProducts(limit int) ([]models.Product, error) { args := m.Called(limit); return args.Get(0).([]models.Product), args.Error(1) }
func (m *MockDashboardRepository) GetUsersOverTime(start, end time.Time) ([]models.TimeSeriesData, error) { args := m.Called(start, end); return args.Get(0).([]models.TimeSeriesData), args.Error(1) }
func (m *MockDashboardRepository) GetUsersWithPurchasesOverTime(start, end time.Time) ([]models.TimeSeriesData, error) { args := m.Called(start, end); return args.Get(0).([]models.TimeSeriesData), args.Error(1) }
