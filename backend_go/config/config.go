package config

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"

	"github.com/joho/godotenv"
)

type Settings struct {
	DatabaseHost             string
	DatabasePort             string
	DatabaseUser             string
	DatabasePassword         string
	DatabaseName             string
	CorsOrigins              []string
	SecretKey                string
	Algorithm                string
	AccessTokenExpireMinutes int
	ServiceAPIKey            string
	Port                     string
}

func (s *Settings) GetDBConnStr() string {
	return fmt.Sprintf(
		"host=%s user=%s password=%s dbname=%s port=%s sslmode=disable",
		s.DatabaseHost, s.DatabaseUser, s.DatabasePassword, s.DatabaseName, s.DatabasePort,
	)
}

func LoadConfig(path string) (Settings, error) {
	var appSettings Settings
	err := godotenv.Load(path)
	if err != nil {
		return appSettings, err
	}

	appSettings.DatabaseHost = os.Getenv("DATABASE_HOST")
	appSettings.DatabasePort = os.Getenv("DATABASE_PORT")
	appSettings.DatabaseUser = os.Getenv("DATABASE_USER")
	appSettings.DatabasePassword = os.Getenv("DATABASE_PASSWORD")
	appSettings.DatabaseName = os.Getenv("DATABASE_NAME")
	appSettings.SecretKey = os.Getenv("SECRET_KEY")
	appSettings.Algorithm = os.Getenv("ALGORITHM")
	appSettings.ServiceAPIKey = os.Getenv("SERVICE_API_KEY")

	expireMinutes, err := strconv.Atoi(os.Getenv("ACCESS_TOKEN_EXPIRE_MINUTES"))
	if err != nil {
		return appSettings, err
	}
	appSettings.AccessTokenExpireMinutes = expireMinutes
	appSettings.Port = os.Getenv("PORT")

	var corsOrigins []string
	err = json.Unmarshal([]byte(os.Getenv("CORS_ORIGINS")), &corsOrigins)
	if err != nil {
		return appSettings, err
	}
	appSettings.CorsOrigins = corsOrigins

	return appSettings, nil
}
