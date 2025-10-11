package handlers

import (
	"frbktg/backend_go/apperrors"
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

	if err := h.userService.ToggleBlockUser(user.TelegramID); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, responses.MessageResponse{Message: "User block status updated successfully"})
}

func (h *AdminHandler) GetUsersHandler(c *gin.Context) {
	users, err := h.userService.GetUsers()
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, users)
}
