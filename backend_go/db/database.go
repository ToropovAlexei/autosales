package db

import (
	"frbktg/backend_go/config"

	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func InitDB(appSettings config.Settings) (*gorm.DB, error) {
	var err error
	var db *gorm.DB
	db, err = gorm.Open(postgres.Open(appSettings.GetDBConnStr()), &gorm.Config{})
	if err != nil {
		return nil, err
	}
	return db, nil
}
