package models

import "time"

type Bot struct {
	ID                 uint      `gorm:"primaryKey"`
	OwnerID            *uint     `gorm:"index"` // Nullable, for referral bots
	Token              string    `gorm:"unique"`
	Username           string    `gorm:"unique"`
	Type               string    `gorm:"index"` // 'main' or 'referral'
	IsActive           bool      `gorm:"default:true"`
	IsPrimary          bool      `gorm:"default:false"` // For referral bots
	ReferralPercentage float64   `gorm:"default:0"`     // For referral bots
	CreatedAt          time.Time `gorm:"autoCreateTime"`
}

type BotResponse struct {
	ID                 uint    `json:"id"`
	Token              string  `json:"token"`
	Username           string  `json:"username"`
	Type               string  `json:"type"`
	IsPrimary          bool    `json:"is_primary"`
	IsActive           bool    `json:"is_active"`
	OwnerID            *uint   `json:"owner_id"`
	OwnerTelegramID    int64   `json:"owner_telegram_id,omitempty"`
	Turnover           float64 `json:"turnover"`
	Accruals           float64 `json:"accruals"`
	ReferralPercentage float64 `json:"referral_percentage"`
}
