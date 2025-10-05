package main

import (
	"fmt"
	"log"

	"frbktg/backend_go/config"
	"frbktg/backend_go/db"
	"frbktg/backend_go/models"
	"gorm.io/gorm"
)

func main() {
	appSettings, err := config.LoadConfig(".env")
	if err != nil {
		log.Fatalf("could not load config: %v", err)
	}

	db, err := db.InitDB(appSettings)
	if err != nil {
		log.Fatalf("could not initialize database: %v", err)
	}

	fmt.Println("Dropping all tables...")
	dropAllTables(db)

	fmt.Println("Auto-migrating tables...")
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
		&models.UserSubscription{},
		&models.PaymentInvoice{},
	); migrateErr != nil {
		log.Fatalf("failed to migrate database: %v", migrateErr)
	}

	fmt.Println("Database initialization completed successfully!")
}

func dropAllTables(db *gorm.DB) {
	tables := []interface{}{
		&models.UserSubscription{},
		&models.RefTransaction{},
		&models.ReferralBot{},
		&models.StockMovement{},
		&models.Order{},
		&models.Transaction{},
		&models.Product{},
		&models.Category{},
		&models.BotUser{},
		&models.User{},
		&models.PaymentInvoice{},
	}
	if err := db.Migrator().DropTable(tables...); err != nil {
		log.Fatalf("Failed to drop tables: %v", err)
	}
}