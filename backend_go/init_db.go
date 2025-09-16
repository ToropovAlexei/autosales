package main

import (
	"log"

	"frbktg/backend_go/config"
	"frbktg/backend_go/db"
	"frbktg/backend_go/models"
)

func main() {
	if err := config.LoadConfig(".env.example"); err != nil {
		log.Fatalf("could not load config: %v", err)
	}

	if err := db.InitDB(); err != nil {
		log.Fatalf("could not initialize database: %v", err)
	}

	db.DB.AutoMigrate(
		&models.User{},
		&models.Category{},
		&models.Product{},
		&models.BotUser{},
		&models.Transaction{},
		&models.Order{},
		&models.StockMovement{},
		&models.ReferralBot{},
		&models.RefTransaction{},
	)
}
