package middleware

import (
	"time"

	"github.com/gin-gonic/gin"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

// LogContext добавляет в контекст запроса логгер и логирует сам запрос после его выполнения.
func LogContext() gin.HandlerFunc {
	return func(c *gin.Context) {
		start := time.Now()

		// Создаем дочерний логгер для этого конкретного запроса
		logger := log.With().
			Str("method", c.Request.Method).
			Str("path", c.Request.URL.Path).
			Str("ip", c.ClientIP()).
			Str("user_agent", c.Request.UserAgent()).
			Logger()

		if rawQuery := c.Request.URL.RawQuery; rawQuery != "" {
			logger = logger.With().Str("query", rawQuery).Logger()
		}

		// Сохраняем обогащенный логгер в контексте Gin
		c.Set("logger", &logger)

		c.Next()

		// После выполнения запроса логируем результат
		latency := time.Since(start)
		status := c.Writer.Status()

		var logEvent *zerolog.Event
		if status >= 500 {
			logEvent = logger.Error()
		} else if status >= 400 {
			logEvent = logger.Warn()
		} else {
			logEvent = logger.Info()
		}

		logEvent.Int("status", status).
			Dur("latency", latency).
			Int("body_size", c.Writer.Size()).
			Send()
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
