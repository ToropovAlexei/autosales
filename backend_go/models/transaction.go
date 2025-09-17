package models

import (
	"time"
)

type TransactionType string

const (
	Deposit  TransactionType = "deposit"
	Purchase TransactionType = "purchase"
)

type Transaction struct {
	ID          uint `gorm:"primaryKey"`
	UserID      uint
	OrderID     *uint
	Type        TransactionType `gorm:"not null"`
	Amount      float64         `gorm:"not null"`
	CreatedAt   time.Time       `gorm:"not null"`
	Description string
}

type TransactionResponse struct {
	ID          uint            `json:"id"`
	UserID      uint            `json:"user_id"`
	OrderID     *uint           `json:"order_id"`
	Type        TransactionType `json:"type"`
	Amount      float64         `json:"amount"`
	CreatedAt   time.Time       `json:"created_at"`
	Description string          `json:"description"`
}
