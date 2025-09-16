package main

import (
	"log"

	"frbktg/backend_go/config"
	"frbktg/backend_go/db"
	"frbktg/backend_go/models"
	"golang.org/x/crypto/bcrypt"
)

func main() {
	if err := config.LoadConfig(".env.example"); err != nil {
		log.Fatalf("could not load config: %v", err)
	}

	if err := db.InitDB(); err != nil {
		log.Fatalf("could not initialize database: %v", err)
	}

	hashedPassword, err := bcrypt.GenerateFromPassword([]byte("password"), bcrypt.DefaultCost)
	if err != nil {
		log.Fatalf("could not hash password: %v", err)
	}

	user := models.User{
		Email:          "test@example.com",
		HashedPassword: string(hashedPassword),
		Role:           models.Admin,
	}

	db.DB.Create(&user)
}
