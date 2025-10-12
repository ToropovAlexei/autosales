package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"

	"github.com/gin-gonic/gin"
)

type SettingHandler struct {
	service *services.SettingService
}

func NewSettingHandler(service *services.SettingService) *SettingHandler {
	return &SettingHandler{service: service}
}

// GetSettings godoc
// @Summary Get all settings
// @Description Get all store settings
// @Tags Admin
// @Produce json
// @Success      200 {object} responses.ResponseSchema[map[string]string]
// @Router /admin/settings [get]
// @Security BearerAuth
func (h *SettingHandler) GetSettings(c *gin.Context) {
	settings, err := h.service.GetSettings()
	if err != nil {
		c.Error(apperrors.New(http.StatusInternalServerError, "Failed to get settings", err))
		return
	}
	responses.SuccessResponse(c, http.StatusOK, settings)
}

// UpdateSettings godoc
// @Summary Update settings
// @Description Update store settings
// @Tags Admin
// @Accept json
// @Produce json
// @Param settings body map[string]string true "Settings map"
// @Success      200 {object} responses.ResponseSchema[responses.MessageResponse]
// @Router /admin/settings [put]
// @Security BearerAuth
func (h *SettingHandler) UpdateSettings(c *gin.Context) {
	var settingsMap map[string]string
	if err := c.ShouldBindJSON(&settingsMap); err != nil {
		c.Error(apperrors.New(http.StatusBadRequest, "Invalid request body", err))
		return
	}

	if err := h.service.UpdateSettings(c, settingsMap); err != nil {
		c.Error(apperrors.New(http.StatusInternalServerError, "Failed to update settings", err))
		return
	}

	responses.SuccessResponse(c, http.StatusOK, responses.MessageResponse{Message: "Settings updated successfully"})
}

// GetPublicSettings godoc
// @Summary Get public settings
// @Description Get public store settings
// @Tags Public
// @Produce json
// @Success      200 {object} responses.ResponseSchema[map[string]string]
// @Router /settings/public [get]
func (h *SettingHandler) GetPublicSettings(c *gin.Context) {
	settings, err := h.service.GetPublicSettings()
	if err != nil {
		c.Error(apperrors.New(http.StatusInternalServerError, "Failed to get settings", err))
		return
	}
	responses.SuccessResponse(c, http.StatusOK, settings)
}
