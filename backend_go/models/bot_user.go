package models

import "time"

type BotUser struct {
	ID               uint      `gorm:"primaryKey"`
	TelegramID       int64     `gorm:"uniqueIndex"`
	IsDeleted        bool      `gorm:"default:false"`
	HasPassedCaptcha bool      `gorm:"default:false"`
	CreatedAt        time.Time `gorm:"not null;default:CURRENT_TIMESTAMP"`
}

type BotUserResponse struct {
	ID               uint    `json:"id"`
	TelegramID       int64   `json:"telegram_id"`
	IsDeleted        bool    `json:"is_deleted"`
	HasPassedCaptcha bool    `json:"has_passed_captcha"`
	Balance          float64 `json:"balance"`
	CreatedAt        time.Time `json:"created_at"`
}
