package handlers

import (
	"bytes"
	"encoding/json"
	"frbktg/backend_go/models"
	"frbktg/backend_go/services/mocks"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)



func TestUserHandler_GetMeHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockUserService := new(mocks.MockUserService)
		user := models.User{ID: 1, Email: "test@test.com"}
		userResponse := &models.UserResponse{ID: 1, Email: "test@test.com"}

		mockUserService.On("GetMe", user).Return(userResponse)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)
		c.Set("user", user)

		h := NewUserHandler(mockUserService)
		h.GetMeHandler(c)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockUserService.AssertExpectations(t)
	})

	t.Run("User not in context", func(t *testing.T) {
		mockUserService := new(mocks.MockUserService)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)

		h := NewUserHandler(mockUserService)
		h.GetMeHandler(c)

		ErrorHandler()(c)

		assert.Equal(t, http.StatusForbidden, rr.Code)
	})
}

func TestUserHandler_UpdateReferralSettingsHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockUserService := new(mocks.MockUserService)
		user := models.User{ID: 1, Role: models.Admin}

		mockUserService.On("UpdateReferralSettings", &user, true, 10.0).Return(nil)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)
		c.Set("user", user)

		payload := map[string]interface{}{"referral_program_enabled": true, "referral_percentage": 10.0}
		body, _ := json.Marshal(payload)
		req, _ := http.NewRequest(http.MethodPut, "/referral-settings", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		c.Request = req

		h := NewUserHandler(mockUserService)
		h.UpdateReferralSettingsHandler(c)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockUserService.AssertExpectations(t)
	})
}

func TestUserHandler_RegisterBotUserHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockUserService := new(mocks.MockUserService)
		telegramID := int64(12345)
		user := &models.BotUser{ID: 1, TelegramID: telegramID}

		mockUserService.On("RegisterBotUser", telegramID).Return(user, 0.0, true, false, nil)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)

		payload := map[string]interface{}{"telegram_id": telegramID}
		body, _ := json.Marshal(payload)
		req, _ := http.NewRequest(http.MethodPost, "/register-bot-user", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		c.Request = req

		h := NewUserHandler(mockUserService)
		h.RegisterBotUserHandler(c)

		assert.Equal(t, http.StatusCreated, rr.Code)
		mockUserService.AssertExpectations(t)
	})
}

func TestUserHandler_GetBotUserHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockUserService := new(mocks.MockUserService)
		userID := uint(1)
		user := &models.BotUser{ID: userID, TelegramID: 12345}

		mockUserService.On("GetBotUser", userID).Return(user, 100.0, nil)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)
		c.Params = []gin.Param{{Key: "id", Value: "1"}}

		h := NewUserHandler(mockUserService)
		h.GetBotUserHandler(c)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockUserService.AssertExpectations(t)
	})
}

func TestUserHandler_GetBalanceHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockUserService := new(mocks.MockUserService)
		telegramID := int64(12345)

		mockUserService.On("GetUserBalance", telegramID).Return(100.0, nil)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)
		c.Params = []gin.Param{{Key: "id", Value: "12345"}}

		h := NewUserHandler(mockUserService)
		h.GetBalanceHandler(c)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockUserService.AssertExpectations(t)
	})
}

func TestUserHandler_GetUserTransactionsHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockUserService := new(mocks.MockUserService)
		telegramID := int64(12345)
		transactions := []models.Transaction{{ID: 1, UserID: 1, Amount: 10.0}}

		mockUserService.On("GetUserTransactions", telegramID).Return(transactions, nil)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)
		c.Params = []gin.Param{{Key: "id", Value: "12345"}}

		h := NewUserHandler(mockUserService)
		h.GetUserTransactionsHandler(c)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockUserService.AssertExpectations(t)
	})
}

func TestUserHandler_UpdateUserCaptchaStatusHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockUserService := new(mocks.MockUserService)
		userID := uint(1)

		mockUserService.On("UpdateUserCaptchaStatus", userID, true).Return(nil)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)
		c.Params = []gin.Param{{Key: "id", Value: "1"}}

		payload := map[string]interface{}{"has_passed_captcha": true}
		body, _ := json.Marshal(payload)
		req, _ := http.NewRequest(http.MethodPut, "/users/1/captcha", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		c.Request = req

		h := NewUserHandler(mockUserService)
		h.UpdateUserCaptchaStatusHandler(c)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockUserService.AssertExpectations(t)
	})
}

func TestUserHandler_GetSellerSettingsHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	t.Run("Success", func(t *testing.T) {
		mockUserService := new(mocks.MockUserService)
		seller := &models.User{ID: 1, ReferralProgramEnabled: true, ReferralPercentage: 10.0}

		mockUserService.On("GetSellerSettings").Return(seller, nil)

		rr := httptest.NewRecorder()
		c, _ := gin.CreateTestContext(rr)

		h := NewUserHandler(mockUserService)
		h.GetSellerSettingsHandler(c)

		assert.Equal(t, http.StatusOK, rr.Code)
		mockUserService.AssertExpectations(t)
	})
}