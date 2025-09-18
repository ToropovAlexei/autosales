package services

import (
	"frbktg/backend_go/models"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"gorm.io/gorm"
)

// --- Tests ---

func TestUserService_RegisterBotUser_NewUser(t *testing.T) {
	// Arrange
	mockUserRepo := new(MockUserRepository)
	mockBotUserRepo := new(MockBotUserRepository)
	userService := NewUserService(mockUserRepo, mockBotUserRepo)

	telegramID := int64(12345)

	mockBotUserRepo.On("FindByTelegramID", telegramID).Return(nil, gorm.ErrRecordNotFound)
	mockBotUserRepo.On("Create", mock.AnythingOfType("*models.BotUser")).Return(nil)

	// Act
	user, balance, isNew, hasPassedCaptcha, err := userService.RegisterBotUser(telegramID)

	// Assert
	assert.NoError(t, err)
	assert.True(t, isNew)
	assert.False(t, hasPassedCaptcha)
	assert.Equal(t, 0.0, balance)
	assert.NotNil(t, user)
	assert.Equal(t, telegramID, user.TelegramID)
	mockBotUserRepo.AssertExpectations(t)
}

func TestUserService_RegisterBotUser_ExistingUser(t *testing.T) {
	// Arrange
	mockUserRepo := new(MockUserRepository)
	mockBotUserRepo := new(MockBotUserRepository)
	userService := NewUserService(mockUserRepo, mockBotUserRepo)

	telegramID := int64(12345)
	existingUser := &models.BotUser{ID: 1, TelegramID: telegramID, IsDeleted: false, HasPassedCaptcha: true}

	mockBotUserRepo.On("FindByTelegramID", telegramID).Return(existingUser, nil)
	mockBotUserRepo.On("GetUserBalance", existingUser.ID).Return(100.50, nil)

	// Act
	user, balance, isNew, hasPassedCaptcha, err := userService.RegisterBotUser(telegramID)

	// Assert
	assert.NoError(t, err)
	assert.False(t, isNew)
	assert.True(t, hasPassedCaptcha)
	assert.Equal(t, 100.50, balance)
	assert.Equal(t, existingUser, user)
	mockBotUserRepo.AssertExpectations(t)
}

func TestUserService_RegisterBotUser_ReactivatedUser(t *testing.T) {
	// Arrange
	mockUserRepo := new(MockUserRepository)
	mockBotUserRepo := new(MockBotUserRepository)
	userService := NewUserService(mockUserRepo, mockBotUserRepo)

	telegramID := int64(12345)
	existingUser := &models.BotUser{ID: 1, TelegramID: telegramID, IsDeleted: true, HasPassedCaptcha: true}

	mockBotUserRepo.On("FindByTelegramID", telegramID).Return(existingUser, nil)
	mockBotUserRepo.On("Update", mock.AnythingOfType("*models.BotUser")).Run(func(args mock.Arguments) {
		arg := args.Get(0).(*models.BotUser)
		assert.False(t, arg.IsDeleted)
		assert.False(t, arg.HasPassedCaptcha)
	}).Return(nil)

	// Act
	user, balance, isNew, hasPassedCaptcha, err := userService.RegisterBotUser(telegramID)

	// Assert
	assert.NoError(t, err)
	assert.True(t, isNew) // isNew is true for reactivated user as per service logic
	assert.False(t, hasPassedCaptcha)
	assert.Equal(t, 0.0, balance)
	assert.NotNil(t, user)
	mockBotUserRepo.AssertExpectations(t)
}

func TestUserService_GetBotUser_NotFound(t *testing.T) {
	// Arrange
	mockUserRepo := new(MockUserRepository)
	mockBotUserRepo := new(MockBotUserRepository)
	userService := NewUserService(mockUserRepo, mockBotUserRepo)

	userID := uint(99)
	mockBotUserRepo.On("FindByID", userID).Return(nil, gorm.ErrRecordNotFound)

	// Act
	user, balance, err := userService.GetBotUser(userID)

	// Assert
	assert.Error(t, err)
	assert.Nil(t, user)
	assert.Equal(t, 0.0, balance)
	mockBotUserRepo.AssertExpectations(t)
}
