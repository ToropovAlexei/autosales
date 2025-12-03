package services

import (
	"fmt"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/google/uuid"
)

type TokenService interface {
	GenerateToken(user *models.User) (string, string, time.Time, error)
	GenerateTemporaryToken(email string) (string, error)
	ValidateToken(tokenString string) (*jwt.Token, error)
}

type tokenService struct {
	secretKey       string
	expireMinutes   int
	activeTokenRepo repositories.ActiveTokenRepository
}

func NewTokenService(secretKey string, expireMinutes int, activeTokenRepo repositories.ActiveTokenRepository) TokenService {
	return &tokenService{
		secretKey:       secretKey,
		expireMinutes:   expireMinutes,
		activeTokenRepo: activeTokenRepo,
	}
}

func (s *tokenService) GenerateTemporaryToken(email string) (string, error) {
	expirationTime := time.Now().Add(5 * time.Minute)
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"sub": email,
		"exp": expirationTime.Unix(),
	})

	return token.SignedString([]byte(s.secretKey))
}


func (s *tokenService) GenerateToken(user *models.User) (string, string, time.Time, error) {
	jti := uuid.New().String()
	expirationTime := time.Now().Add(time.Minute * time.Duration(s.expireMinutes))
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"sub": user.Email,
		"exp": expirationTime.Unix(),
		"jti": jti,
	})

	tokenString, err := token.SignedString([]byte(s.secretKey))
	return tokenString, jti, expirationTime, err
}

func (s *tokenService) ValidateToken(tokenString string) (*jwt.Token, error) {
	token, err := jwt.Parse(tokenString, func(token *jwt.Token) (interface{}, error) {
		if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
		return []byte(s.secretKey), nil
	})

	if err != nil {
		return nil, err
	}

	if claims, ok := token.Claims.(jwt.MapClaims); ok && token.Valid {
		jti, ok := claims["jti"].(string)
		if !ok {
			return nil, fmt.Errorf("jti claim not found")
		}

		exists, err := s.activeTokenRepo.Exists(jti)
		if err != nil {
			return nil, err
		}
		if !exists {
			return nil, fmt.Errorf("token is not active")
		}
	}

	return token, nil
}
