package models

import "time"

type ActiveToken struct {
	JTI       string `gorm:"primaryKey"`
	UserID    uint
	ExpiresAt time.Time
}
