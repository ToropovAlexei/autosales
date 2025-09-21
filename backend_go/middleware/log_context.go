package middleware

import (
	"github.com/gin-gonic/gin"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

// LogContext добавляет в контекст запроса логгер с полями, специфичными для этого запроса.
func LogContext() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Создаем дочерний логгер для этого конкретного запроса
		logger := log.With().
			Str("method", c.Request.Method).
			Str("path", c.Request.URL.Path).
			Str("ip", c.ClientIP()).
			Str("user_agent", c.Request.UserAgent()).
			Logger()

		// Сохраняем обогащенный логгер в контексте Gin
		c.Set("logger", &logger)

		c.Next()
	}
}

// GetLogger извлекает обогащенный логгер из контекста Gin.
// Если логгер не найден, возвращает глобальный логгер.
func GetLogger(c *gin.Context) *zerolog.Logger {
	if l, exists := c.Get("logger"); exists {
		if logger, ok := l.(*zerolog.Logger); ok {
			return logger
		}
	}
	// Fallback на глобальный логгер
	return &log.Logger
}
