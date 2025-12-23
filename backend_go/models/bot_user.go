package models

import "time"

type BotUser struct {
	ID                 uint      `gorm:"primaryKey" json:"id"`
	TelegramID         int64     `gorm:"uniqueIndex" json:"telegram_id"`
	Balance            float64   `gorm:"default:0" json:"balance"`
	IsBlocked          bool      `gorm:"default:false" json:"is_blocked"`
	BotIsBlockedByUser bool      `gorm:"default:false" json:"bot_is_blocked_by_user"`
	HasPassedCaptcha   bool      `gorm:"default:false" json:"has_passed_captcha"`
	RegisteredWithBot  string    `gorm:"size:255" json:"registered_with_bot"`
	LastSeenWithBot    string    `gorm:"size:255" json:"last_seen_with_bot"`
	LastSeenAt         time.Time `json:"last_seen_at"`
	CreatedAt          time.Time `gorm:"not null;default:CURRENT_TIMESTAMP" json:"created_at"`
}
