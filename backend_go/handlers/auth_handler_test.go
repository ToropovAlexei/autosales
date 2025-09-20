package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/services/mocks"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

func TestAuthHandler_LoginHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockAuthService := new(mocks.MockAuthService)
		mockAuthService.On("Login", "test@test.com", "password123").Return("test-token", nil)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)

		h := NewAuthHandler(mockAuthService)
		router.POST("/login", h.LoginHandler)

		form := url.Values{}
		form.Add("username", "test@test.com")
		form.Add("password", "password123")

		req, _ := http.NewRequest(http.MethodPost, "/login", strings.NewReader(form.Encode()))
		req.Header.Add("Content-Type", "application/x-www-form-urlencoded")

		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockAuthService.AssertExpectations(t)
	})

	t.Run("Invalid Payload", func(t *testing.T) {
		mockAuthService := new(mocks.MockAuthService)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)
		router.Use(func(c *gin.Context) {
			c.Next()
			if len(c.Errors) > 0 {
				c.JSON(400, gin.H{"error": c.Errors.String()})
			}
		})

		h := NewAuthHandler(mockAuthService)
		router.POST("/login", h.LoginHandler)

		form := url.Values{}
		form.Add("username", "test@test.com")

		req, _ := http.NewRequest(http.MethodPost, "/login", strings.NewReader(form.Encode()))
		req.Header.Add("Content-Type", "application/x-www-form-urlencoded")

		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusBadRequest, rr.Code)
	})

	t.Run("Login Fails", func(t *testing.T) {
		mockAuthService := new(mocks.MockAuthService)
		mockAuthService.On("Login", "test@test.com", "wrong-password").Return("", apperrors.ErrUnauthorized)

		rr := httptest.NewRecorder()
		_, router := gin.CreateTestContext(rr)
		router.Use(func(c *gin.Context) {
			c.Next()
			if len(c.Errors) > 0 {
				c.JSON(401, gin.H{"error": c.Errors.String()})
			}
		})

		h := NewAuthHandler(mockAuthService)
		router.POST("/login", h.LoginHandler)

		form := url.Values{}
		form.Add("username", "test@test.com")
		form.Add("password", "wrong-password")

		req, _ := http.NewRequest(http.MethodPost, "/login", strings.NewReader(form.Encode()))
		req.Header.Add("Content-Type", "application/x-www-form-urlencoded")

		router.ServeHTTP(rr, req)

		assert.Equal(t, http.StatusUnauthorized, rr.Code)
		mockAuthService.AssertExpectations(t)
	})
}
