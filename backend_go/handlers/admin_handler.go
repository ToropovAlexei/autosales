package handlers

import (
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

type AdminHandler struct {
	adminService services.AdminService
}

func NewAdminHandler(adminService services.AdminService) *AdminHandler {
	return &AdminHandler{adminService: adminService}
}

func (h *AdminHandler) GetBotUsersHandler(c *gin.Context) {
	users, err := h.adminService.GetBotUsersWithBalance()
	if err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, err.Error())
		return
	}
	responses.SuccessResponse(c, http.StatusOK, users)
}

func (h *AdminHandler) DeleteBotUserHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		responses.ErrorResponse(c, http.StatusBadRequest, "Invalid user ID")
		return
	}

	if err := h.adminService.SoftDeleteBotUser(uint(id)); err != nil {
		responses.ErrorResponse(c, http.StatusNotFound, err.Error())
		return
	}

	c.Status(http.StatusNoContent)
}
