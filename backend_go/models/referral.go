package models

import "time"

type RefTransaction struct {
	ID         uint `gorm:"primaryKey"`
	RefOwnerID uint
	OrderID    uint
	Amount     float64   `gorm:"not null"`
	RefShare   float64   `gorm:"not null"`
	CreatedAt  time.Time `gorm:"autoCreateTime"`
}

type ReferralBotStats struct {
	BotID         uint    `json:"bot_id"`
	TotalEarnings float64 `json:"total_earnings"`
	PurchaseCount int64   `json:"purchase_count"`
}
