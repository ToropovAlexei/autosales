package middleware

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

func BotUserAuthMiddleware(userService services.UserService) gin.HandlerFunc {
	return func(c *gin.Context) {
		telegramIDStr := c.GetHeader("X-Telegram-ID")
		if telegramIDStr == "" {
			c.Error(apperrors.New(http.StatusUnauthorized, "X-Telegram-ID header is missing", nil))
			c.Abort()
			return
		}

		telegramID, err := strconv.ParseInt(telegramIDStr, 10, 64)
		if err != nil {
			c.Error(apperrors.New(http.StatusUnauthorized, "Invalid X-Telegram-ID header", err))
			c.Abort()
			return
		}

		botUser, err := userService.GetBotUserByTelegramID(telegramID, "") // botName is not used here
		if err != nil {
			c.Error(apperrors.New(http.StatusUnauthorized, "Bot user not found", err))
			c.Abort()
			return
		}

		c.Set("botUser", botUser)
		c.Next()
	}
}
