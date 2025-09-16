package db

import (
	"frbktg/backend_go/config"

	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

var DB *gorm.DB

func InitDB() (err error) {
	DB, err = gorm.Open(postgres.Open(config.AppSettings.GetDBConnStr()), &gorm.Config{})
	return
}

func GetDB() *gorm.DB {
	return DB
}
