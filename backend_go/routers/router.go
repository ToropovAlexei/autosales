package routers

import (
	"frbktg/backend_go/config"
	"gorm.io/gorm"
	"log/slog"
)

type Router struct {
	db          *gorm.DB
	appSettings config.Settings
	logger      *slog.Logger
}

func NewRouter(db *gorm.DB, appSettings config.Settings, logger *slog.Logger) *Router {
	return &Router{db: db, appSettings: appSettings, logger: logger}
}
