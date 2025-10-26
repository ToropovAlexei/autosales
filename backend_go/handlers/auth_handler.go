package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
)

type AuthHandler struct {
	authService services.AuthService
}

func NewAuthHandler(authService services.AuthService) *AuthHandler {
	return &AuthHandler{authService: authService}
}

type LoginPayload struct {
	Email    string `json:"email" binding:"required,email"`
	Password string `json:"password" binding:"required"`
}

type Verify2FAPayload struct {
	TempToken string `json:"temp_token" binding:"required"`
	Code      string `json:"code" binding:"required"`
}

// @Summary      User Login
// @Description  Logs in a user and returns a temporary token if 2FA is required
// @Tags         Auth
// @Accept       json
// @Produce      json
// @Param        login body LoginPayload true "Login credentials"
// @Success      200  {object}  responses.ResponseSchema[responses.TokenResponse]
// @Failure      400  {object}  responses.ErrorResponseSchema
// @Failure      401  {object}  responses.ErrorResponseSchema
// @Router       /auth/login [post]
func (h *AuthHandler) LoginHandler(c *gin.Context) {
	var payload LoginPayload
	if err := c.ShouldBindJSON(&payload); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	token, is2FARequired, err := h.authService.Login(payload.Email, payload.Password)
	if err != nil {
		c.Error(err)
		return
	}

	if is2FARequired {
		responses.SuccessResponse(c, http.StatusOK, gin.H{"tfa_required": true, "temp_token": token})
		return
	}

	responses.SuccessResponse(c, http.StatusOK, responses.TokenResponse{AccessToken: token, TokenType: "bearer"})
}

// @Summary      Verify 2FA
// @Description  Verifies the 2FA code and returns a JWT token
// @Tags         Auth
// @Accept       json
// @Produce      json
// @Param        login body Verify2FAPayload true "2FA verification data"
// @Success      200  {object}  responses.ResponseSchema[responses.TokenResponse]
// @Failure      400  {object}  responses.ErrorResponseSchema
// @Failure      401  {object}  responses.ErrorResponseSchema
// @Router       /auth/verify-2fa [post]
func (h *AuthHandler) Verify2FAHandler(c *gin.Context) {
	var payload Verify2FAPayload
	if err := c.ShouldBindJSON(&payload); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	tokenString, err := h.authService.Verify2FA(payload.TempToken, payload.Code)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, responses.TokenResponse{AccessToken: tokenString, TokenType: "bearer"})
}

// @Summary      User Logout
// @Description  Logs out a user and invalidates their JWT token
// @Tags         Auth
// @Success      204
// @Failure      401  {object}  responses.ErrorResponseSchema
// @Router       /auth/logout [post]
// @Security     ApiKeyAuth
func (h *AuthHandler) LogoutHandler(c *gin.Context) {
	authHeader := c.GetHeader("Authorization")
	if authHeader == "" {
		c.Error(apperrors.New(http.StatusUnauthorized, "Authorization header is missing", nil))
		return
	}

	tokenString := strings.TrimPrefix(authHeader, "Bearer ")

	parser := jwt.Parser{}
	token, _, err := parser.ParseUnverified(tokenString, jwt.MapClaims{})
	if err != nil {
		c.Error(apperrors.New(http.StatusUnauthorized, "Invalid token", err))
		return
	}

	if claims, ok := token.Claims.(jwt.MapClaims); ok {
		jti, ok := claims["jti"].(string)
		if !ok {
			c.Error(apperrors.New(http.StatusUnauthorized, "jti claim not found", nil))
			return
		}

		if err := h.authService.Logout(jti); err != nil {
			c.Error(err)
			return
		}
	}

	responses.SuccessResponse(c, http.StatusNoContent, nil)
}
