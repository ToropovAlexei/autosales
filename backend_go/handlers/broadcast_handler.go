package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/models"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"

	"github.com/gin-gonic/gin"
)

type BroadcastHandler struct {
	broadcastService services.BroadcastService
}

func NewBroadcastHandler(broadcastService services.BroadcastService) *BroadcastHandler {
	return &BroadcastHandler{
		broadcastService: broadcastService,
	}
}

// @Summary      Get Bot Users for Broadcast
// @Description  Retrieves a paginated list of bot users based on specified filter criteria.
// @Tags         Admin, Broadcasts
// @Produce      json
// @Param        page query int false "Page number" default(1)
// @Param        pageSize query int false "Page size" default(10)
// @Param        balance_min query number false "Minimum balance"
// @Param        balance_max query number false "Maximum balance"
// @Param        registered_after query string false "Registered after date (RFC3339)"
// @Param        registered_before query string false "Registered before date (RFC3339)"
// @Param        last_seen_after query string false "Last seen after date (RFC3339)"
// @Param        last_seen_before query string false "Last seen before date (RFC3339)"
// @Param        bot_name query string false "Bot name"
// @Success      200 {object} responses.ResponseSchema[models.PaginatedResult[models.BotUser]]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Router       /admin/broadcasts/users [get]
// @Security     ApiKeyAuth
func (h *BroadcastHandler) GetUsers(c *gin.Context) {
	var page models.Page
	if err := c.ShouldBindQuery(&page); err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid pagination query: " + err.Error()})
		return
	}

	var filters models.BroadcastFilters
	if err := c.ShouldBindQuery(&filters); err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid filters query: " + err.Error()})
		return
	}

	users, err := h.broadcastService.GetFilteredUsers(filters, page)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, users)
}

// @Summary      Send Broadcast
// @Description  Initiates a broadcast to a filtered group of users. This is an async operation.
// @Tags         Admin, Broadcasts
// @Accept       json
// @Produce      json
// @Param        payload body models.BroadcastPayload true "Broadcast content and user filters"
// @Success      202 {object} responses.ResponseSchema[responses.MessageResponse]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Router       /admin/broadcasts/send [post]
// @Security     ApiKeyAuth
func (h *BroadcastHandler) SendBroadcast(c *gin.Context) {
	var payload models.BroadcastPayload
	if err := c.ShouldBindJSON(&payload); err != nil {
		c.Error(&apperrors.ErrValidation{Message: "Invalid payload: " + err.Error()})
		return
	}

	if err := h.broadcastService.StartBroadcast(payload); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusAccepted, responses.MessageResponse{Message: "Broadcast accepted for processing."})
}

