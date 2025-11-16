package main

import (
	"fmt"
	"log"

	"frbktg/backend_go/config"
	"frbktg/backend_go/db"
	"frbktg/backend_go/models"
	"gorm.io/gorm"
)

func dropAllTables(db *gorm.DB) {
	tables := []interface{}{
		&models.UserSubscription{}, &models.RefTransaction{}, &models.Bot{},
		&models.StockMovement{}, &models.Order{}, &models.Transaction{},
		&models.Product{}, &models.Category{}, &models.BotUser{}, &models.User{},
		&models.PaymentInvoice{}, &models.Image{}, &models.UserPermission{},
		&models.UserRole{}, &models.RolePermission{}, &models.Permission{}, &models.Role{},
		&models.ActiveToken{},
		&models.TemporaryToken{},
		&models.StoreBalance{},
	}
	if err := db.Migrator().DropTable(tables...); err != nil {
		log.Fatalf("Failed to drop tables: %v", err)
	}
}

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
		&models.Bot{},
		&models.RefTransaction{},
		&models.UserSubscription{},
		&models.PaymentInvoice{},
		&models.Image{},
		&models.Role{},
		&models.Permission{},
		&models.RolePermission{},
		&models.UserRole{},
		&models.UserPermission{},
		&models.Setting{},
		&models.ActiveToken{},
		&models.TemporaryToken{},
		&models.StoreBalance{},
	); migrateErr != nil {
		log.Fatalf("failed to migrate database: %v", migrateErr)
	}

	fmt.Println("Seeding initial settings...")
	initialSettings := []models.Setting{
		{Key: "GLOBAL_PRICE_MARKUP", Value: "0"},
		{Key: "GATEWAY_COMMISSION_mock_provider", Value: "0"},
		{Key: "GATEWAY_COMMISSION_platform_card", Value: "0"},
		{Key: "GATEWAY_COMMISSION_platform_sbp", Value: "0"},
	}
	if err := db.Create(&initialSettings).Error; err != nil {
		log.Fatalf("failed to seed settings: %v", err)
	}

	fmt.Println("Database initialization completed successfully!")
}