package responses

import (
	"frbktg/backend_go/models"

	"github.com/gin-gonic/gin"
)

// ResponseSchema defines the generic structure for API responses.
// @Description This is the standard API response structure.
type ResponseSchema[T any] struct {
	Success bool    `json:"success" example:"true"`
	Data    *T      `json:"data,omitempty"`
	Error   *string `json:"error,omitempty" extensions:"x-nullable=true" example:"null"`
}

// ErrorResponseSchema defines the structure for a failed API response, used for documentation.
// @Description This is the standard error response structure.
type ErrorResponseSchema struct {
	Success bool         `json:"success" example:"false"`
	Data    *interface{} `json:"data"`
	Error   *string      `json:"error"`
}

type TokenResponse struct {
	AccessToken string `json:"access_token"`
	TokenType   string `json:"token_type"`
}

type MessageResponse struct {
	Message string `json:"message"`
}

type RegisterBotUserResponse struct {
	User             models.BotUserResponse `json:"user"`
	IsNew            bool                   `json:"is_new"`
	HasPassedCaptcha bool                   `json:"has_passed_captcha"`
}

type BalanceResponse struct {
	Balance float64 `json:"balance"`
}

type SellerSettingsResponse struct {
	ID                     uint    `json:"id"`
	ReferralProgramEnabled bool    `json:"referral_program_enabled"`
	ReferralPercentage     float64 `json:"referral_percentage"`
}

func ErrorResponse(c *gin.Context, statusCode int, message string) {
	c.JSON(statusCode, gin.H{"success": false, "data": nil, "error": message})
}

func SuccessResponse(c *gin.Context, statusCode int, data interface{}) {
	c.JSON(statusCode, gin.H{"success": true, "data": data, "error": nil})
}
