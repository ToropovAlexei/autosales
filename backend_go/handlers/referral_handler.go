package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

type StatsHandler struct {
	referralService services.ReferralService
}

func NewStatsHandler(referralService services.ReferralService) *StatsHandler {
	return &StatsHandler{referralService: referralService}
}

func (h *StatsHandler) GetReferralStatsHandler(c *gin.Context) {
	telegramID, err := strconv.ParseInt(c.Param("telegram_id"), 10, 64)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid Telegram ID"})
		return
	}

	stats, err := h.referralService.GetReferralStats(telegramID)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, stats)
}
