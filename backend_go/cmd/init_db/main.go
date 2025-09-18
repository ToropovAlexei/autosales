package main

import (
	"log"

	"frbktg/backend_go/config"
	"frbktg/backend_go/db"
	"frbktg/backend_go/models"
)

func main() {
	appSettings, err := config.LoadConfig("../../.env.example")
	if err != nil {
		log.Fatalf("could not load config: %v", err)
	}

	db, err := db.InitDB(appSettings)
	if err != nil {
		log.Fatalf("could not initialize database: %v", err)
	}

	if migrateErr := db.AutoMigrate(
		&models.User{},
		&models.Category{},
		&models.Product{},
		&models.BotUser{},
		&models.Transaction{},
		&models.Order{},
		&models.StockMovement{},
		&models.ReferralBot{},
		&models.RefTransaction{},
	); migrateErr != nil {
		log.Fatalf("failed to migrate database: %v", migrateErr)
	}
}
