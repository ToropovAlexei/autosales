package config

import (
	"io"
	"os"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"gopkg.in/natefinch/lumberjack.v2"
)

// InitLogger инициализирует логгер zerolog с ротацией файлов.
func InitLogger() {
	// Настраиваем lumberjack для ротации логов в файле
	logRotator := &lumberjack.Logger{
		Filename:   "server.log", // Имя файла логов
		MaxSize:    10,           // Максимальный размер файла в мегабайтах
		MaxBackups: 5,            // Максимальное количество старых файлов логов
		MaxAge:     30,           // Максимальное количество дней хранения старых файлов
		Compress:   true,         // Сжимать старые файлы логов
	}

	// Создаем ConsoleWriter для красивого вывода в консоль
	consoleWriter := zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: time.RFC3339}

	// MultiWriter для одновременной записи в файл (в формате JSON) и в консоль (в текстовом формате)
	multiWriter := io.MultiWriter(consoleWriter, logRotator)

	// Устанавливаем глобальный логгер
	log.Logger = zerolog.New(multiWriter).With().Timestamp().Logger()

	log.Info().Msg("Logger initialized successfully")
}
