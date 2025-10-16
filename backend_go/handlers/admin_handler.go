package handlers

import (
	"encoding/json"
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

type AdminHandler struct {
	adminService services.AdminService
	userService  services.UserService
}

func NewAdminHandler(adminService services.AdminService, userService services.UserService) *AdminHandler {
	return &AdminHandler{adminService: adminService, userService: userService}
}

func (h *AdminHandler) GetBotUsers(c *gin.Context) {
	users, err := h.adminService.GetBotUsersWithBalance()
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, users)
}

func (h *AdminHandler) GetBotUser(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid user ID"})
		return
	}

	user, balance, err := h.userService.GetBotUser(uint(id))
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{"user": user, "balance": balance})
}

func (h *AdminHandler) ToggleBlockUser(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid user ID"})
		return
	}

	// First, get the user to find their telegram_id
	user, _, err := h.userService.GetBotUser(uint(id))
	if err != nil {
		c.Error(err)
		return
	}

	if err := h.userService.ToggleBlockUser(c, user.TelegramID); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, responses.MessageResponse{Message: "User block status updated successfully"})
}

func (h *AdminHandler) GetUsersHandler(c *gin.Context) {
	var page models.Page
	if err := c.ShouldBindQuery(&page); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	var filters []models.Filter
	if filtersJSON := c.Query("filters"); filtersJSON != "" {
		if err := json.Unmarshal([]byte(filtersJSON), &filters); err != nil {
			c.Error(&apperrors.ErrValidation{Message: "Invalid filters format"})
			return
		}
	}

	users, err := h.userService.GetUsers(page, filters)
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, users)
}

type createUserPayload struct {
	Email    string `json:"email" binding:"required,email"`
	Password string `json:"password" binding:"required,min=8"`
	RoleID   uint   `json:"role_id" binding:"required"`
}

func (h *AdminHandler) CreateUserHandler(c *gin.Context) {
	var json createUserPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Message: err.Error()})
		return
	}

	user, err := h.userService.CreateUser(c, json.Email, json.Password, json.RoleID)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusCreated, user)
}
