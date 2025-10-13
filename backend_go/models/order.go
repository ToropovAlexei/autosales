package models

import "time"

type Order struct {
	ID            uint      `gorm:"primaryKey" json:"id"`
	UserID        uint      `json:"user_id"`
	ProductID     uint      `json:"product_id"`
	Product       Product   `gorm:"foreignKey:ProductID" json:"Product"`
	Quantity      int       `gorm:"default:1" json:"quantity"`
	Amount        float64   `json:"amount"`
	Status        string    `json:"status"`
	ReferralBotID *uint     `gorm:"index" json:"referral_bot_id"`
	CreatedAt     time.Time `gorm:"not null;default:now()" json:"created_at"`
}

type OrderResponse struct {
	ID             uint      `json:"id"`
	UserID         uint      `json:"user_id"`
	ProductID      uint      `json:"product_id"`
	Quantity       int       `json:"quantity"`
	Amount         float64   `json:"amount"`
	Status         string    `json:"status"`
	CreatedAt      time.Time `json:"created_at"`
	UserTelegramID int64     `json:"user_telegram_id"`
	ProductName    string    `json:"product_name"`
}

type OrderSlimResponse struct {
	ID        uint      `json:"id"`
	UserID    uint      `json:"user_id"`
	ProductID uint      `json:"product_id"`
	Quantity  int       `json:"quantity"`
	Amount    float64   `json:"amount"`
	Status    string    `json:"status"`
	CreatedAt time.Time `json:"created_at"`
}
