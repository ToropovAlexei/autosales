package models

import "time"

type BotUser struct {
	ID                uint      `gorm:"primaryKey"`
	TelegramID        int64     `gorm:"uniqueIndex"`
	IsDeleted         bool      `gorm:"default:false"`
	HasPassedCaptcha  bool      `gorm:"default:false"`
	RegisteredWithBot string    `gorm:"size:255"`
	LastSeenWithBot   string    `gorm:"size:255"`
	LastSeenAt        time.Time
	CreatedAt         time.Time `gorm:"not null;default:CURRENT_TIMESTAMP"`
}

type BotUserResponse struct {
	ID                uint      `json:"id"`
	TelegramID        int64     `json:"telegram_id"`
	IsDeleted         bool      `json:"is_deleted"`
	HasPassedCaptcha  bool      `json:"has_passed_captcha"`
	Balance           float64   `json:"balance"`
	RegisteredWithBot string    `json:"registered_with_bot"`
	LastSeenWithBot   string    `json:"last_seen_with_bot"`
	LastSeenAt        time.Time `json:"last_seen_at"`
	CreatedAt         time.Time `json:"created_at"`
}
