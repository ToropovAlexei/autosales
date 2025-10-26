package models

import "time"

type TemporaryToken struct {
	Token     string `gorm:"primaryKey"`
	UserID    uint
	ExpiresAt time.Time
}
