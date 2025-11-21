package config

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"

	"github.com/rs/zerolog/log"
)

type Config struct {
	DatabaseHost                  string
	DatabasePort                  string
	DatabaseUser                  string
	DatabasePassword              string
	DatabaseName                  string
	CorsOrigins                   []string
	SecretKey                     string
	TFASecretKey                  string
	Algorithm                     string
	AccessTokenExpireMinutes      int
	ServiceAPIKey                 string
	Port                          string
	MockGatewayURL                string
	BotDispatcherWebhookURL       string
	PaymentNotificationMinutes    int
	ImageUploadPath               string
	MainBotName                   string
	MainBotToken                  string
	FallbackBotName               string
	FallbackBotToken              string
	PlatformPaymentSystemBaseURL  string
	PlatformPaymentSystemLogin    string
	PlatformPaymentSystemPassword string
	PlatformPaymentSystem2FAKey   string
}

func (c *Config) GetDBConnStr() string {
	return fmt.Sprintf(
		"host=%s user=%s password=%s dbname=%s port=%s sslmode=disable",
		c.DatabaseHost, c.DatabaseUser, c.DatabasePassword, c.DatabaseName, c.DatabasePort,
	)
}

func LoadConfig() (*Config, error) {
	cfg := &Config{
		DatabaseHost:                  os.Getenv("DATABASE_HOST"),
		DatabasePort:                  os.Getenv("DATABASE_PORT"),
		DatabaseUser:                  os.Getenv("DATABASE_USER"),
		DatabasePassword:              os.Getenv("DATABASE_PASSWORD"),
		DatabaseName:                  os.Getenv("DATABASE_NAME"),
		SecretKey:                     os.Getenv("SECRET_KEY"),
		TFASecretKey:                  os.Getenv("TFA_SECRET_KEY"),
		Algorithm:                     os.Getenv("ALGORITHM"),
		ServiceAPIKey:                 os.Getenv("SERVICE_API_KEY"),
		Port:                          os.Getenv("PORT"),
		MockGatewayURL:                os.Getenv("MOCK_GATEWAY_URL"),
		BotDispatcherWebhookURL:       os.Getenv("BOT_DISPATCHER_WEBHOOK_URL"),
		ImageUploadPath:               os.Getenv("IMAGE_UPLOAD_PATH"),
		MainBotName:                   os.Getenv("MAIN_BOT_NAME"),
		MainBotToken:                  os.Getenv("MAIN_BOT_TOKEN"),
		FallbackBotName:               os.Getenv("FALLBACK_BOT_NAME"),
		FallbackBotToken:              os.Getenv("FALLBACK_BOT_TOKEN"),
		PlatformPaymentSystemBaseURL:  os.Getenv("PLATFORM_PAYMENT_SYSTEM_BASE_URL"),
		PlatformPaymentSystemLogin:    os.Getenv("PLATFORM_PAYMENT_SYSTEM_LOGIN"),
		PlatformPaymentSystemPassword: os.Getenv("PLATFORM_PAYMENT_SYSTEM_PASSWORD"),
		PlatformPaymentSystem2FAKey:   os.Getenv("PLATFORM_PAYMENT_SYSTEM_2FA_KEY"),
	}

	// AccessTokenExpireMinutes
	if v := os.Getenv("ACCESS_TOKEN_EXPIRE_MINUTES"); v != "" {
		if val, err := strconv.Atoi(v); err == nil {
			cfg.AccessTokenExpireMinutes = val
		} else {
			log.Warn().Msgf("Invalid ACCESS_TOKEN_EXPIRE_MINUTES: %s", v)
		}
	}

	// PaymentNotificationMinutes
	if v := os.Getenv("PAYMENT_NOTIFICATION_MINUTES"); v != "" {
		if val, err := strconv.Atoi(v); err == nil {
			cfg.PaymentNotificationMinutes = val
		} else {
			log.Warn().Msgf("Invalid PAYMENT_NOTIFICATION_MINUTES: %s", v)
		}
	}

	// CORS_ORIGINS
	corsOriginsStr := os.Getenv("CORS_ORIGINS")
	if corsOriginsStr != "" {
		var corsOrigins []string
		if err := json.Unmarshal([]byte(corsOriginsStr), &corsOrigins); err != nil {
			log.Warn().Err(err).Msg("Failed to parse CORS_ORIGINS, ignoring")
		} else {
			cfg.CorsOrigins = corsOrigins
		}
	}

	// Лог всех основных переменных для проверки
	log.Info().Msgf("Config loaded: %+v", cfg)
	return cfg, nil
}
