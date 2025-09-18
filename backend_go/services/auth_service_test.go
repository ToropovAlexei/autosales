package services

import (
	"errors"
	"frbktg/backend_go/config"
	"frbktg/backend_go/models"
	"testing"

	"github.com/stretchr/testify/assert"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

func setupAuthServiceTest() (*authService, *MockUserRepository, *MockTokenService) {
	userRepo := new(MockUserRepository)
	tokenService := new(MockTokenService)
	appSettings := config.Settings{SecretKey: "test-secret", AccessTokenExpireMinutes: 15}

	sut := NewAuthService(userRepo, tokenService, appSettings).(*authService)
	return sut, userRepo, tokenService
}

func TestAuthService_Login_Success(t *testing.T) {
	// Arrange
	sut, userRepo, tokenService := setupAuthServiceTest()

	hashedPassword, _ := bcrypt.GenerateFromPassword([]byte("password123"), bcrypt.DefaultCost)
	user := &models.User{Email: "test@test.com", HashedPassword: string(hashedPassword)}

	userRepo.On("FindByEmail", "test@test.com").Return(user, nil)
	tokenService.On("GenerateToken", user, "test-secret", 15).Return("test-token", nil)

	// Act
	token, err := sut.Login("test@test.com", "password123")

	// Assert
	assert.NoError(t, err)
	assert.Equal(t, "test-token", token)
	userRepo.AssertExpectations(t)
	tokenService.AssertExpectations(t)
}

func TestAuthService_Login_UserNotFound(t *testing.T) {
	// Arrange
	sut, userRepo, _ := setupAuthServiceTest()

	userRepo.On("FindByEmail", "wrong@test.com").Return(nil, gorm.ErrRecordNotFound)

	// Act
	token, err := sut.Login("wrong@test.com", "password123")

	// Assert
	assert.Error(t, err)
	assert.Empty(t, token)
	assert.Equal(t, "incorrect username or password", err.Error())
	userRepo.AssertExpectations(t)
}

func TestAuthService_Login_WrongPassword(t *testing.T) {
	// Arrange
	sut, userRepo, _ := setupAuthServiceTest()

	hashedPassword, _ := bcrypt.GenerateFromPassword([]byte("password123"), bcrypt.DefaultCost)
	user := &models.User{Email: "test@test.com", HashedPassword: string(hashedPassword)}

	userRepo.On("FindByEmail", "test@test.com").Return(user, nil)

	// Act
	token, err := sut.Login("test@test.com", "wrong-password")

	// Assert
	assert.Error(t, err)
	assert.Empty(t, token)
	assert.Equal(t, "incorrect username or password", err.Error())
	userRepo.AssertExpectations(t)
}

func TestAuthService_Login_TokenGenerationFails(t *testing.T) {
	// Arrange
	sut, userRepo, tokenService := setupAuthServiceTest()

	hashedPassword, _ := bcrypt.GenerateFromPassword([]byte("password123"), bcrypt.DefaultCost)
	user := &models.User{Email: "test@test.com", HashedPassword: string(hashedPassword)}

	userRepo.On("FindByEmail", "test@test.com").Return(user, nil)
	tokenService.On("GenerateToken", user, "test-secret", 15).Return("", errors.New("token error"))

	// Act
	token, err := sut.Login("test@test.com", "password123")

	// Assert
	assert.Error(t, err)
	assert.Empty(t, token)
	assert.Equal(t, "token error", err.Error())
	userRepo.AssertExpectations(t)
	tokenService.AssertExpectations(t)
}
