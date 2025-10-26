package services

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/config"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"time"

	"github.com/google/uuid"
	"golang.org/x/crypto/bcrypt"
)

type AuthService interface {
	Login(email, password string) (string, bool, error)
	Verify2FA(tempToken, code string) (string, error)
	Logout(jti string) error
}

type authService struct {
	userRepo           repositories.UserRepository
	tokenService       TokenService
	twoFAService       TwoFAService
	activeTokenRepo    repositories.ActiveTokenRepository
	temporaryTokenRepo repositories.TemporaryTokenRepository
	appSettings        config.Settings
}

func NewAuthService(userRepo repositories.UserRepository, tokenService TokenService, twoFAService TwoFAService, activeTokenRepo repositories.ActiveTokenRepository, temporaryTokenRepo repositories.TemporaryTokenRepository, appSettings config.Settings) AuthService {
	return &authService{
		userRepo:           userRepo,
		tokenService:       tokenService,
		twoFAService:       twoFAService,
		activeTokenRepo:    activeTokenRepo,
		temporaryTokenRepo: temporaryTokenRepo,
		appSettings:        appSettings,
	}
}

func (s *authService) Login(email, password string) (string, bool, error) {
	user, err := s.userRepo.FindByEmail(email)
	if err != nil {
		return "", false, &apperrors.ErrValidation{Message: "incorrect username or password"}
	}

	if err := bcrypt.CompareHashAndPassword([]byte(user.HashedPassword), []byte(password)); err != nil {
		return "", false, &apperrors.ErrValidation{Message: "incorrect username or password"}
	}

	if user.TwoFAEnabled {
		tempToken := uuid.New().String()
		if err := s.temporaryTokenRepo.Create(&models.TemporaryToken{
			Token:     tempToken,
			UserID:    user.ID,
			ExpiresAt: time.Now().Add(5 * time.Minute),
		}); err != nil {
			return "", false, err
		}
		return tempToken, true, nil
	}

	tokenString, jti, expirationTime, err := s.tokenService.GenerateToken(user)
	if err != nil {
		return "", false, err
	}

	if err := s.activeTokenRepo.Create(jti, user.ID, expirationTime); err != nil {
		return "", false, err
	}

	return tokenString, false, nil
}

func (s *authService) Verify2FA(tempToken, code string) (string, error) {
	tempTokenData, err := s.temporaryTokenRepo.Find(tempToken)
	if err != nil {
		return "", &apperrors.ErrValidation{Message: "invalid or expired temporary token"}
	}

	if time.Now().After(tempTokenData.ExpiresAt) {
		return "", &apperrors.ErrValidation{Message: "invalid or expired temporary token"}
	}

	if err := s.temporaryTokenRepo.Delete(tempToken); err != nil {
		// Log the error, but continue with the login
	}

	user, err := s.userRepo.FindByID(tempTokenData.UserID)
	if err != nil {
		return "", &apperrors.ErrValidation{Message: "user not found"}
	}

	decryptedSecret, err := s.twoFAService.DecryptSecret(*user.TwoFASecret)
	if err != nil {
		return "", err
	}

	if !s.twoFAService.ValidateCode(decryptedSecret, code) {
		return "", &apperrors.ErrValidation{Message: "invalid 2FA code"}
	}

	tokenString, jti, expirationTime, err := s.tokenService.GenerateToken(user)
	if err != nil {
		return "", err
	}

	if err := s.activeTokenRepo.Create(jti, user.ID, expirationTime); err != nil {
		return "", err
	}

	return tokenString, nil
}

func (s *authService) Logout(jti string) error {
	return s.activeTokenRepo.Delete(jti)
}
