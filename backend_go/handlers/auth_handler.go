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
	Username string `form:"username" binding:"required"`
	Password string `form:"password" binding:"required"`
}

// @Summary      User Login
// @Description  Logs in a user and returns a JWT token
// @Tags         Auth
// @Accept       x-www-form-urlencoded
// @Produce      json
// @Param        username formData string true "Username (Email)"
// @Param        password formData string true "Password"
// @Success      200  {object}  map[string]string
// @Failure      400  {object}  map[string]string
// @Failure      401  {object}  map[string]string
// @Router       /auth/login [post]
func (h *AuthHandler) LoginHandler(c *gin.Context) {
	var form loginPayload

	if err := c.ShouldBind(&form); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	tokenString, err := h.authService.Login(form.Username, form.Password)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{"access_token": tokenString, "token_type": "bearer"})
}