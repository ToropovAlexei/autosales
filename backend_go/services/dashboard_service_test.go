package services

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestDashboardService_GetDashboardStats(t *testing.T) {
	// Arrange
	mockRepo := new(MockDashboardRepository)
	dashboardService := NewDashboardService(mockRepo)

	mockRepo.On("CountTotalUsers").Return(int64(100), nil)
	mockRepo.On("CountUsersWithPurchases").Return(int64(25), nil)
	mockRepo.On("CountAvailableProducts").Return(int64(10), nil)

	// Act
	stats, err := dashboardService.GetDashboardStats()

	// Assert
	assert.NoError(t, err)
	assert.NotNil(t, stats)
	assert.Equal(t, int64(100), stats.TotalUsers)
	assert.Equal(t, int64(25), stats.UsersWithPurchases)
	assert.Equal(t, int64(10), stats.AvailableProducts)
	mockRepo.AssertExpectations(t)
}

func TestDashboardService_GetDashboardStats_Error(t *testing.T) {
	// Arrange
	mockRepo := new(MockDashboardRepository)
	dashboardService := NewDashboardService(mockRepo)

	expectedError := errors.New("db error")
	mockRepo.On("CountTotalUsers").Return(int64(100), nil)
	mockRepo.On("CountUsersWithPurchases").Return(int64(0), expectedError) // Simulate error on one of the calls
	mockRepo.On("CountAvailableProducts").Return(int64(10), nil)      // This expectation was missing

	// Act
	stats, err := dashboardService.GetDashboardStats()

	// Assert
	assert.Error(t, err)
	assert.Nil(t, stats)
	assert.Equal(t, expectedError, err)
}
