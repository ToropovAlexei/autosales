package middleware

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/config"
	"frbktg/backend_go/models"
	"frbktg/backend_go/services"
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
)

// AuthMiddleware handles authentication.

type AuthMiddleware struct {
	tokenService services.TokenService
	userService  services.UserService
}

// NewAuthMiddleware creates a new AuthMiddleware.
func NewAuthMiddleware(tokenService services.TokenService, userService services.UserService) *AuthMiddleware {
	return &AuthMiddleware{
		tokenService: tokenService,
		userService:  userService,
	}
}

// RequireAuth is a middleware function to protect routes.
func (m *AuthMiddleware) RequireAuth(c *gin.Context) {
	authHeader := c.GetHeader("Authorization")
	if authHeader == "" {
		c.Error(apperrors.New(http.StatusUnauthorized, "Authorization header is missing", nil))
		c.Abort()
		return
	}

	tokenString := strings.TrimPrefix(authHeader, "Bearer ")

	token, err := m.tokenService.ValidateToken(tokenString)

	if err != nil {
		c.Error(apperrors.New(http.StatusUnauthorized, "Invalid token", err))
		c.Abort()
		return
	}

	if claims, ok := token.Claims.(jwt.MapClaims); ok && token.Valid {
		user, err := m.userService.GetMeByEmail(claims["sub"].(string))
		if err != nil {
			c.Error(apperrors.New(http.StatusUnauthorized, "User not found", err))
			c.Abort()
			return
		}

		c.Set("user", *user)
		c.Next()
	} else {
		c.Error(apperrors.New(http.StatusUnauthorized, "Invalid token", nil))
		c.Abort()
		return
	}
}

func ServiceTokenMiddleware(appSettings config.Settings) gin.HandlerFunc {
	return func(c *gin.Context) {
		apiKey := c.GetHeader("X-API-KEY")
		if apiKey == "" {
			c.Error(apperrors.New(http.StatusForbidden, "API key is missing", nil))
			c.Abort()
			return
		}

		if apiKey != appSettings.ServiceAPIKey {
			c.Error(apperrors.New(http.StatusForbidden, "Invalid service token", nil))
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
			c.Error(apperrors.New(http.StatusUnauthorized, "User not found in context", nil))
			c.Abort()
			return
		}

		currentUser, ok := user.(models.User)
		if !ok {
			c.Error(apperrors.New(http.StatusInternalServerError, "Invalid user type in context", nil))
			c.Abort()
			return
		}
		if currentUser.Role != models.Admin {
			c.Error(apperrors.New(http.StatusForbidden, "The user doesn't have enough privileges", nil))
			c.Abort()
			return
		}

		c.Next()
	}
}