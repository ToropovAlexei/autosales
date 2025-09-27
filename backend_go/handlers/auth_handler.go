package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"

	"github.com/gin-gonic/gin"
)

type AuthHandler struct {
	authService services.AuthService
}

func NewAuthHandler(authService services.AuthService) *AuthHandler {
	return &AuthHandler{authService: authService}
}

type loginPayload struct {
	Email    string `json:"email" binding:"required"`
	Password string `json:"password" binding:"required"`
}

// @Summary      User Login
// @Description  Logs in a user and returns a JWT token
// @Tags         Auth
// @Accept       json
// @Produce      json
// @Param        login body loginPayload true "Login credentials"
// @Success      200  {object}  responses.ResponseSchema[responses.TokenResponse]
// @Failure      400  {object}  responses.ErrorResponseSchema
// @Failure      401  {object}  responses.ErrorResponseSchema
// @Router       /auth/login [post]
func (h *AuthHandler) LoginHandler(c *gin.Context) {
	var payload loginPayload

	if err := c.ShouldBindJSON(&payload); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	tokenString, err := h.authService.Login(payload.Email, payload.Password)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, responses.TokenResponse{AccessToken: tokenString, TokenType: "bearer"})
}