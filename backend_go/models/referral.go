package models

import "time"

type ReferralBot struct {
	ID                 uint      `gorm:"primaryKey"`
	OwnerID            uint      `gorm:"index"`
	BotToken           string    `gorm:"unique"`
	IsActive           bool      `gorm:"default:true"`
	IsPrimary          bool      `gorm:"default:false"`
	ReferralPercentage float64   `gorm:"default:0"`
	CreatedAt          time.Time `gorm:"autoCreateTime"`
}

type RefTransaction struct {
	ID         uint `gorm:"primaryKey"`
	RefOwnerID uint
	OrderID    uint
	Amount     float64   `gorm:"not null"`
	RefShare   float64   `gorm:"not null"`
	CreatedAt  time.Time `gorm:"autoCreateTime"`
}

type ReferralBotAdminInfo struct {
	ID                 uint      `json:"id"`
	OwnerID            uint      `json:"owner_id"`
	BotToken           string    `json:"bot_token"`
	IsActive           bool      `json:"is_active"`
	IsPrimary          bool      `json:"is_primary"`
	ReferralPercentage float64   `json:"referral_percentage"`
	CreatedAt          time.Time `json:"created_at"`
	OwnerTelegramID    int64     `json:"owner_telegram_id"`
	Turnover           float64   `json:"turnover"`
	Accruals           float64   `json:"accruals"`
}

type ReferralBotResponse struct {
	ID        uint      `json:"id"`
	OwnerID   uint      `json:"owner_id"`
	BotToken  string    `json:"bot_token"`
	IsActive  bool      `json:"is_active"`
	IsPrimary bool      `json:"is_primary"`
	CreatedAt time.Time `json:"created_at"`
}
