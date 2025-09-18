package services

import (
	"errors"
	"frbktg/backend_go/config"
	"frbktg/backend_go/repositories"

	"golang.org/x/crypto/bcrypt"
)

type AuthService interface {
	Login(username, password string) (string, error)
}

type authService struct {
	userRepo      repositories.UserRepository
	tokenService  TokenService
	appSettings   config.Settings
}

func NewAuthService(userRepo repositories.UserRepository, tokenService TokenService, appSettings config.Settings) AuthService {
	return &authService{userRepo: userRepo, tokenService: tokenService, appSettings: appSettings}
}

func (s *authService) Login(username, password string) (string, error) {
	user, err := s.userRepo.FindByEmail(username)
	if err != nil {
		return "", errors.New("incorrect username or password")
	}

	if err := bcrypt.CompareHashAndPassword([]byte(user.HashedPassword), []byte(password)); err != nil {
		return "", errors.New("incorrect username or password")
	}

	return s.tokenService.GenerateToken(user, s.appSettings.SecretKey, s.appSettings.AccessTokenExpireMinutes)
}
