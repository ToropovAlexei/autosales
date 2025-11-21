package main

import (
	"errors"
	"flag"
	"frbktg/backend_go/config"
	"frbktg/backend_go/db"
	"frbktg/backend_go/models"

	"github.com/rs/zerolog/log"
	"gorm.io/gorm"
)

func main() {
	config.InitLogger()

	flag.Parse()

	appSettings, err := config.LoadConfig()
	if err != nil {
		log.Fatal().Err(err).Msg("could not load config")
	}

	db, err := db.InitDB(appSettings)
	if err != nil {
		log.Fatal().Err(err).Msg("failed to create db connection")
	}

	log.Info().Msg("Starting backfill of balances...")

	var users []models.BotUser
	if err := db.Find(&users).Error; err != nil {
		log.Fatal().Err(err).Msg("failed to get users")
	}

	for _, user := range users {
		var balance float64
		if err := db.Model(&models.Transaction{}).Where("user_id = ?", user.ID).Select("sum(amount)").Row().Scan(&balance); err != nil {
			if !errors.Is(err, gorm.ErrRecordNotFound) {
				log.Error().Err(err).Uint("user_id", user.ID).Msg("failed to calculate balance")
				continue
			}
		}

		if err := db.Model(&models.BotUser{}).Where("id = ?", user.ID).Update("balance", balance).Error; err != nil {
			log.Error().Err(err).Uint("user_id", user.ID).Msg("failed to update balance")
		}

		log.Info().Uint("user_id", user.ID).Float64("balance", balance).Msg("updated balance")
	}

	log.Info().Msg("Backfill of balances completed.")
}
