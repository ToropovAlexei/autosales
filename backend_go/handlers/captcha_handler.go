package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"github.com/mojocn/base64Captcha"
)

type CaptchaHandler struct {
	// No dependencies needed for this simple captcha generation
}

func NewCaptchaHandler() *CaptchaHandler {
	return &CaptchaHandler{}
}

// @Summary      Generate Captcha
// @Description  Generates a base64 encoded captcha image and its answer.
// @Tags         Public
// @Produce      json
// @Param        height query int false "Captcha image height" default(80)
// @Param        width query int false "Captcha image width" default(240)
// @Param        length query int false "Captcha character length" default(6)
// @Success      200 {object} responses.ResponseSchema[map[string]string]
// @Failure      400 {object} responses.ErrorResponseSchema
// @Failure      500 {object} responses.ErrorResponseSchema
// @Router       /captcha [get]
func (h *CaptchaHandler) GetCaptchaHandler(c *gin.Context) {
	// Default values
	height := 80
	width := 240
	length := 6

	// Parse query parameters
	if hStr := c.Query("height"); hStr != "" {
		if h, err := strconv.Atoi(hStr); err == nil && h > 0 {
			height = h
		}
	}
	if wStr := c.Query("width"); wStr != "" {
		if w, err := strconv.Atoi(wStr); err == nil && w > 0 {
			width = w
		}
	}
	if lStr := c.Query("length"); lStr != "" {
		if l, err := strconv.Atoi(lStr); err == nil && l > 0 {
			length = l
		}
	}

	// Configure captcha
	source := "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
	driver := base64Captcha.NewDriverString(height, width, 0, base64Captcha.OptionShowHollowLine, length, source, nil, nil, nil)
	captcha := base64Captcha.NewCaptcha(driver, base64Captcha.DefaultMemStore)

	// Generate captcha
	_, content, answer, err := captcha.Generate()
	if err != nil {
		c.Error(apperrors.New(http.StatusInternalServerError, "Failed to generate captcha", err))
		return
	}

	if content == "" {
		c.Error(apperrors.New(http.StatusInternalServerError, "Failed to generate captcha image", nil))
		return
	}

	responses.SuccessResponse(c, http.StatusOK, gin.H{
		"imageData": content,
		"answer":    answer,
	})
}
