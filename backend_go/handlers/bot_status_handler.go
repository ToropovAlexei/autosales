package handlers

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"github.com/gin-gonic/gin"
	"net/http"
)

type BotStatusHandler struct {
	service services.BotStatusService
}

func NewBotStatusHandler(service services.BotStatusService) *BotStatusHandler {
	return &BotStatusHandler{service: service}
}

// GetBotStatus godoc
// @Summary Get bot operational status
// @Description Checks if the bot is allowed to operate based on store balance
// @Tags Bot
// @Produce  json
// @Success 200 {object} responses.ResponseSchema[models.BotStatusResponse]
// @Failure 500 {object} responses.ErrorResponseSchema
// @Router /bot/status [get]
// @Security ServiceApiKeyAuth
func (h *BotStatusHandler) GetBotStatus(c *gin.Context) {
	canOperate, err := h.service.GetBotStatus()
	if err != nil {
		responses.ErrorResponse(c, http.StatusInternalServerError, "Failed to get bot status")
		return
	}

	responses.SuccessResponse(c, http.StatusOK, models.BotStatusResponse{CanOperate: canOperate})
}
