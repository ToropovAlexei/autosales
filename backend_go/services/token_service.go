package services

import (
	"fmt"
	"frbktg/backend_go/models"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

type TokenService interface {
	GenerateToken(user *models.User, secretKey string, expireMinutes int) (string, error)
	ValidateToken(tokenString string, secretKey string) (*jwt.Token, error)
}

type tokenService struct{}

func NewTokenService() TokenService {
	return &tokenService{}
}

func (s *tokenService) GenerateToken(user *models.User, secretKey string, expireMinutes int) (string, error) {
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"sub":  user.Email,
		"role": user.Role,
		"exp":  time.Now().Add(time.Minute * time.Duration(expireMinutes)).Unix(),
	})

	return token.SignedString([]byte(secretKey))
}

func (s *tokenService) ValidateToken(tokenString string, secretKey string) (*jwt.Token, error) {
	return jwt.Parse(tokenString, func(token *jwt.Token) (interface{}, error) {
		if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
		return []byte(secretKey), nil
	})
}
