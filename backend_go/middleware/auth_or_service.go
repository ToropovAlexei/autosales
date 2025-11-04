package middleware

import (
	"frbktg/backend_go/config"
	"frbktg/backend_go/repositories"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
)

// AuthOrServiceTokenMiddleware проверяет наличие либо валидного JWT токена пользователя, либо сервисного ключа.
// Если сервисный ключ есть и он валиден, запрос пропускается.
// Если его нет, проверяется JWT токен.
// Если ни один из методов аутентификации не проходит, возвращается ошибка 401.
func AuthOrServiceTokenMiddleware(appSettings *config.Config, tokenService services.TokenService, userRepo repositories.UserRepository) gin.HandlerFunc {
	return func(c *gin.Context) {
		// 1. Проверяем сервисный ключ
		apiKey := c.GetHeader("X-API-KEY")
		if apiKey != "" {
			if apiKey == appSettings.ServiceAPIKey {
				c.Next()
				return
			} else {
				// Если ключ есть, но он неверный - это ошибка.
				responses.ErrorResponse(c, http.StatusForbidden, "Invalid service token")
				c.Abort()
				return
			}
		}

		// 2. Если сервисного ключа нет, проверяем JWT токен пользователя
		authHeader := c.GetHeader("Authorization")
		if authHeader == "" {
			responses.ErrorResponse(c, http.StatusUnauthorized, "Authorization header or API key is missing")
			c.Abort()
			return
		}

		tokenString := strings.TrimPrefix(authHeader, "Bearer ")
		token, err := tokenService.ValidateToken(tokenString)

		if err != nil {
			responses.ErrorResponse(c, http.StatusUnauthorized, "Invalid token")
			c.Abort()
			return
		}

		if claims, ok := token.Claims.(jwt.MapClaims); ok && token.Valid {
			user, err := userRepo.FindByEmail(claims["sub"].(string))
			if err != nil {
				responses.ErrorResponse(c, http.StatusUnauthorized, "User not found")
				c.Abort()
				return
			}

			c.Set("user", *user)
			c.Next()
		} else {
			responses.ErrorResponse(c, http.StatusUnauthorized, "Invalid token")
			c.Abort()
			return
		}
	}
}
