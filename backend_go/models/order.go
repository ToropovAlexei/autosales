package models

import "time"

type Order struct {
	ID        uint `gorm:"primaryKey"`
	UserID    uint
	ProductID uint
	Product   Product
	Quantity  int `gorm:"default:1"`
	Amount    float64
	Status    string
	CreatedAt time.Time `gorm:"not null;default:now()"`
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
