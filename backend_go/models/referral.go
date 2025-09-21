package models

import "time"

type ReferralBot struct {
	ID        uint      `gorm:"primaryKey"`
	OwnerID   uint
	SellerID  uint
	BotToken  string    `gorm:"unique"`
	IsActive  bool      `gorm:"default:true"`
	CreatedAt time.Time `gorm:"autoCreateTime"`
}

type RefTransaction struct {
	ID         uint `gorm:"primaryKey"`
	RefOwnerID uint
	SellerID   uint
	OrderID    uint
	Amount     float64   `gorm:"not null"`
	RefShare   float64   `gorm:"not null"`
	CreatedAt  time.Time `gorm:"autoCreateTime"`
}

type ReferralBotAdminInfo struct {
	ID              uint      `json:"id"`
	OwnerID         uint      `json:"owner_id"`
	SellerID        uint      `json:"seller_id"`
	BotToken        string    `json:"bot_token"`
	IsActive        bool      `json:"is_active"`
	CreatedAt       time.Time `json:"created_at"`
	OwnerTelegramID int64     `json:"owner_telegram_id"`
	Turnover        float64   `json:"turnover"`
	Accruals        float64   `json:"accruals"`
}

type ReferralBotResponse struct {
	ID        uint      `json:"id"`
	OwnerID   uint      `json:"owner_id"`
	SellerID  uint      `json:"seller_id"`
	BotToken  string    `json:"bot_token"`
	IsActive  bool      `json:"is_active"`
	CreatedAt time.Time `json:"created_at"`
}
