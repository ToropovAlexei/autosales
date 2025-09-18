package middleware

import (
	"frbktg/backend_go/config"
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
)

func AuthMiddleware(appSettings config.Settings, tokenService services.TokenService, userRepo repositories.UserRepository) gin.HandlerFunc {
	return func(c *gin.Context) {
		authHeader := c.GetHeader("Authorization")
		if authHeader == "" {
			responses.ErrorResponse(c, http.StatusUnauthorized, "Authorization header is missing")
			c.Abort()
			return
		}

		tokenString := strings.TrimPrefix(authHeader, "Bearer ")

		token, err := tokenService.ValidateToken(tokenString, appSettings.SecretKey)

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

func ServiceTokenMiddleware(appSettings config.Settings) gin.HandlerFunc {
	return func(c *gin.Context) {
		apiKey := c.GetHeader("X-API-KEY")
		if apiKey == "" {
			responses.ErrorResponse(c, http.StatusForbidden, "API key is missing")
			c.Abort()
			return
		}

		if apiKey != appSettings.ServiceAPIKey {
			responses.ErrorResponse(c, http.StatusForbidden, "Invalid service token")
			c.Abort()
			return
		}

		c.Next()
	}
}

func AdminMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		user, exists := c.Get("user")
		if !exists {
			responses.ErrorResponse(c, http.StatusUnauthorized, "User not found in context")
			c.Abort()
			return
		}

		currentUser, ok := user.(models.User)
		if !ok {
			responses.ErrorResponse(c, http.StatusInternalServerError, "Invalid user type in context")
			c.Abort()
			return
		}
		if currentUser.Role != models.Admin {
			responses.ErrorResponse(c, http.StatusForbidden, "The user doesn't have enough privileges")
			c.Abort()
			return
		}

		c.Next()
	}
}