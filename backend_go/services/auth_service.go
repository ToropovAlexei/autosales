package services

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/config"
	"frbktg/backend_go/repositories"

	"golang.org/x/crypto/bcrypt"
)

type AuthService interface {
	Login(username, password string) (string, error)
	Logout(jti string) error
}

type authService struct {
	userRepo        repositories.UserRepository
	tokenService    TokenService
	activeTokenRepo repositories.ActiveTokenRepository
	appSettings     config.Settings
}

func NewAuthService(userRepo repositories.UserRepository, tokenService TokenService, activeTokenRepo repositories.ActiveTokenRepository, appSettings config.Settings) AuthService {
	return &authService{userRepo: userRepo, tokenService: tokenService, activeTokenRepo: activeTokenRepo, appSettings: appSettings}
}

func (s *authService) Login(username, password string) (string, error) {
	user, err := s.userRepo.FindByEmail(username)
	if err != nil {
		return "", &apperrors.ErrValidation{Message: "incorrect username or password"}
	}

	if err := bcrypt.CompareHashAndPassword([]byte(user.HashedPassword), []byte(password)); err != nil {
		return "", &apperrors.ErrValidation{Message: "incorrect username or password"}
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