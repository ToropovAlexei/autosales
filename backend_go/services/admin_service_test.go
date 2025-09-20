package services

import (
	"errors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories/mocks"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestAdminService_GetBotUsersWithBalance(t *testing.T) {
	// Arrange
	mockAdminRepo := new(mocks.MockAdminRepository)
	mockBotUserRepo := new(mocks.MockBotUserRepository)
	adminService := NewAdminService(mockAdminRepo, mockBotUserRepo)

	users := []models.BotUser{
		{ID: 1, TelegramID: 111},
		{ID: 2, TelegramID: 222},
	}
	mockAdminRepo.On("GetActiveBotUsers").Return(users, nil)
	mockBotUserRepo.On("GetUserBalance", uint(1)).Return(100.0, nil)
	mockBotUserRepo.On("GetUserBalance", uint(2)).Return(0.0, nil) // User with zero balance

	// Act
	userResponses, err := adminService.GetBotUsersWithBalance()

	// Assert
	assert.NoError(t, err)
	assert.Len(t, userResponses, 2)
	assert.Equal(t, int64(111), userResponses[0].TelegramID)
	assert.Equal(t, 100.0, userResponses[0].Balance)
	assert.Equal(t, int64(222), userResponses[1].TelegramID)
	assert.Equal(t, 0.0, userResponses[1].Balance)
	mockAdminRepo.AssertExpectations(t)
	mockBotUserRepo.AssertExpectations(t)
}

func TestAdminService_SoftDeleteBotUser(t *testing.T) {
	// Arrange
	mockAdminRepo := new(mocks.MockAdminRepository)
	mockBotUserRepo := new(mocks.MockBotUserRepository)
	adminService := NewAdminService(mockAdminRepo, mockBotUserRepo)

	user := &models.BotUser{ID: 1}
	mockAdminRepo.On("GetBotUserByID", uint(1)).Return(user, nil)
	mockAdminRepo.On("SoftDeleteBotUser", user).Return(nil)

	// Act
	err := adminService.SoftDeleteBotUser(1)

	// Assert
	assert.NoError(t, err)
	mockAdminRepo.AssertExpectations(t)
}

func TestAdminService_SoftDeleteBotUser_NotFound(t *testing.T) {
	// Arrange
	mockAdminRepo := new(mocks.MockAdminRepository)
	mockBotUserRepo := new(mocks.MockBotUserRepository)
	adminService := NewAdminService(mockAdminRepo, mockBotUserRepo)

	mockAdminRepo.On("GetBotUserByID", uint(99)).Return(nil, errors.New("not found"))

	// Act
	err := adminService.SoftDeleteBotUser(99)

	// Assert
	assert.Error(t, err)
	assert.Equal(t, "BotUser with ID 99 not found", err.Error())
	mockAdminRepo.AssertExpectations(t)
}
