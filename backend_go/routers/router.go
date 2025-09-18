package routers

import (
	"frbktg/backend_go/config"
	"frbktg/backend_go/repositories"
	"frbktg/backend_go/services"
	"log/slog"

	"gorm.io/gorm"
)

type Router struct {
	db           *gorm.DB
	appSettings  config.Settings
	logger       *slog.Logger
	tokenService services.TokenService
	userRepo     repositories.UserRepository
}

func NewRouter(db *gorm.DB, appSettings config.Settings, logger *slog.Logger, tokenService services.TokenService, userRepo repositories.UserRepository) *Router {
	return &Router{db: db, appSettings: appSettings, logger: logger, tokenService: tokenService, userRepo: userRepo}
}
