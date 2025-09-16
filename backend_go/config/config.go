package config

import (
	"fmt"
	"os"

	"github.com/joho/godotenv"
)

type Settings struct {
	DATABASE_HOST               string
	DATABASE_PORT               string
	DATABASE_USER               string
	DATABASE_PASSWORD           string
	DATABASE_NAME               string
	CORS_ORIGINS                []string
	SECRET_KEY                  string
	ALGORITHM                   string
	ACCESS_TOKEN_EXPIRE_MINUTES int
	SERVICE_API_KEY             string
}

func (s *Settings) GetDBConnStr() string {
	return fmt.Sprintf("host=%s user=%s password=%s dbname=%s port=%s sslmode=disable", s.DATABASE_HOST, s.DATABASE_USER, s.DATABASE_PASSWORD, s.DATABASE_NAME, s.DATABASE_PORT)
}

var AppSettings Settings

func LoadConfig(path string) (err error) {
	err = godotenv.Load(path)
	if err != nil {
		return
	}

	AppSettings.DATABASE_HOST = os.Getenv("DATABASE_HOST")
	AppSettings.DATABASE_PORT = os.Getenv("DATABASE_PORT")
	AppSettings.DATABASE_USER = os.Getenv("DATABASE_USER")
	AppSettings.DATABASE_PASSWORD = os.Getenv("DATABASE_PASSWORD")
	AppSettings.DATABASE_NAME = os.Getenv("DATABASE_NAME")
	AppSettings.SECRET_KEY = os.Getenv("SECRET_KEY")
	AppSettings.ALGORITHM = os.Getenv("ALGORITHM")
	AppSettings.SERVICE_API_KEY = os.Getenv("SERVICE_API_KEY")

	return
}
