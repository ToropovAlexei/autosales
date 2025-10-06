package config

import (
	"encoding/json"
	"fmt"
	"strings"

	"github.com/spf13/viper"
)

type Settings struct {
	DatabaseHost               string   `mapstructure:"DATABASE_HOST"`
	DatabasePort               string   `mapstructure:"DATABASE_PORT"`
	DatabaseUser               string   `mapstructure:"DATABASE_USER"`
	DatabasePassword           string   `mapstructure:"DATABASE_PASSWORD"`
	DatabaseName               string   `mapstructure:"DATABASE_NAME"`
	CorsOrigins                []string `mapstructure:"CORS_ORIGINS"`
	SecretKey                  string   `mapstructure:"SECRET_KEY"`
	Algorithm                  string   `mapstructure:"ALGORITHM"`
	AccessTokenExpireMinutes   int      `mapstructure:"ACCESS_TOKEN_EXPIRE_MINUTES"`
	ServiceAPIKey              string   `mapstructure:"SERVICE_API_KEY"`
	Port                       string   `mapstructure:"PORT"`
	MockGatewayURL             string   `mapstructure:"MOCK_GATEWAY_URL"`
	BotDispatcherWebhookURL    string   `mapstructure:"BOT_DISPATCHER_WEBHOOK_URL"`
	PaymentNotificationMinutes int      `mapstructure:"PAYMENT_NOTIFICATION_MINUTES"`
}

func (s *Settings) GetDBConnStr() string {
	return fmt.Sprintf(
		"host=%s user=%s password=%s dbname=%s port=%s sslmode=disable",
		s.DatabaseHost, s.DatabaseUser, s.DatabasePassword, s.DatabaseName, s.DatabasePort,
	)
}

func LoadConfig(path string) (Settings, error) {
	var appSettings Settings

	viper.SetConfigFile(path)
	viper.SetConfigType("env")

	viper.AutomaticEnv()
	viper.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))

	if err := viper.ReadInConfig(); err != nil {
		return appSettings, err
	}

	if err := viper.Unmarshal(&appSettings); err != nil {
		return appSettings, err
	}

	corsOriginsStr := viper.GetString("CORS_ORIGINS")
	if corsOriginsStr != "" {
		var corsOrigins []string
		if err := json.Unmarshal([]byte(corsOriginsStr), &corsOrigins); err != nil {
			return appSettings, err
		}
		appSettings.CorsOrigins = corsOrigins
	}

	return appSettings, nil
}
