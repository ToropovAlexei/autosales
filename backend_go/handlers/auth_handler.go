package handlers

import (
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

func (h *AuthHandler) LoginHandler(c *gin.Context) {
	var form loginPayload

	if err := c.ShouldBind(&form); err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, err.Error())
		return
	}

	tokenString, err := h.authService.Login(form.Username, form.Password)
	if err != nil {
		responses.ErrorResponse(c, http.StatusUnauthorized, err.Error())
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{"access_token": tokenString, "token_type": "bearer"})
}
