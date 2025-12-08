package models

import (
	"time"

	"github.com/google/uuid"
)

type Order struct {
	ID               uint       `gorm:"primaryKey" json:"id"`
	UserID           uint       `json:"user_id"`
	ProductID        uint       `json:"product_id"`
	Product          Product    `gorm:"foreignKey:ProductID" json:"Product"`
	Quantity         int        `gorm:"default:1" json:"quantity"`
	Amount           float64    `json:"amount"`
	Status           string     `json:"status"`
	BotID            uint       `gorm:"index" json:"bot_id"`
	FulfilledContent string     `gorm:"type:text" json:"fulfilled_content"`
	FulfilledImageID *uuid.UUID `gorm:"type:uuid" json:"fulfilled_image_id"`
	FulfilledImage   *Image     `gorm:"foreignKey:FulfilledImageID" json:"fulfilled_image"`
	CreatedAt        time.Time  `gorm:"not null;default:now()" json:"created_at"`
}

type OrderResponse struct {
	ID                uint      `json:"id"`
	UserID            uint      `json:"user_id"`
	ProductID         uint      `json:"product_id"`
	Quantity          int       `json:"quantity"`
	Amount            float64   `json:"amount"`
	Status            string    `json:"status"`
	CreatedAt         time.Time `json:"created_at"`
	UserTelegramID    int64     `json:"user_telegram_id"`
	ProductName       string    `json:"product_name"`
	FulfilledContent  string    `json:"fulfilled_content,omitempty"`
	FulfilledImageURL string    `json:"fulfilled_image_url,omitempty"`
	ImageURL          string    `json:"image_url,omitempty"`
}

type OrderSlimResponse struct {
	ID                uint      `json:"id"`
	UserID            uint      `json:"user_id"`
	ProductID         uint      `json:"product_id"`
	Quantity          int       `json:"quantity"`
	Amount            float64   `json:"amount"`
	Status            string    `json:"status"`
	CreatedAt         time.Time `json:"created_at"`
	FulfilledImageURL string    `json:"fulfilled_image_url,omitempty"`
}

// UserOrderResponse is the DTO for orders returned to the bot user.
type UserOrderResponse struct {
	ID                uint      `json:"id"`
	ProductName       string    `json:"product_name"`
	Amount            float64   `json:"amount"`
	CreatedAt         time.Time `json:"created_at"`
	FulfilledContent  string    `json:"fulfilled_content,omitempty"`
	ProductImageURL   string    `json:"product_image_url,omitempty"`
	FulfilledImageURL string    `json:"fulfilled_image_url,omitempty"`
}
