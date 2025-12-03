package middleware

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/config"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
)

// AuthMiddleware handles authentication.

type AuthMiddleware struct {
	tokenService services.TokenService
	userService  services.UserService
	appSettings  *config.Config
}

// NewAuthMiddleware creates a new AuthMiddleware.
func NewAuthMiddleware(tokenService services.TokenService, userService services.UserService, appSettings *config.Config) *AuthMiddleware {
	return &AuthMiddleware{
		tokenService: tokenService,
		userService:  userService,
		appSettings:  appSettings,
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

func ServiceTokenMiddleware(appSettings *config.Config) gin.HandlerFunc {
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

// BotAdminAuthMiddleware authenticates a bot admin user based on their Telegram ID passed in a header.
func (m *AuthMiddleware) BotAdminAuthMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// 1. Check for service API key first
		ServiceTokenMiddleware(m.appSettings)(c)
		if c.IsAborted() {
			return
		}

		// 2. Get Admin Telegram ID from header
		adminIDHeader := c.GetHeader("X-Admin-Telegram-ID")
		if adminIDHeader == "" {
			c.Error(apperrors.New(http.StatusForbidden, "X-Admin-Telegram-ID header is missing", nil))
			c.Abort()
			return
		}

		adminID, err := strconv.ParseInt(adminIDHeader, 10, 64)
		if err != nil {
			c.Error(apperrors.New(http.StatusBadRequest, "Invalid X-Admin-Telegram-ID header", err))
			c.Abort()
			return
		}

		// 3. Find user by Telegram ID
		user, err := m.userService.FindByTelegramID(adminID)
		if err != nil {
			c.Error(apperrors.New(http.StatusForbidden, "Admin user with given Telegram ID not found or not linked", err))
			c.Abort()
			return
		}

		// 4. Set user in context for subsequent permission checks
		c.Set("user", *user)
		c.Next()
	}
}