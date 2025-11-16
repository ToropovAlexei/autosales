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
	ID                 uint      `gorm:"primaryKey"`
	UserID             uint      `gorm:"index"`
	User               BotUser   `gorm:"foreignKey:UserID"`
	OrderID            *uint
	Type               TransactionType `gorm:"not null"`
	Amount             float64         `gorm:"not null"`
	CreatedAt          time.Time       `gorm:"not null"`
	Description        string
	PaymentGateway     string  `gorm:"size:255"`
	GatewayCommission  float64
	PlatformCommission float64
	StoreBalanceDelta  float64
}

type TransactionResponse struct {
	ID                 uint            `json:"id"`
	UserID             uint            `json:"user_id"`
	OrderID            *uint           `json:"order_id"`
	Type               TransactionType `json:"type"`
	Amount             float64         `json:"amount"`
	CreatedAt          time.Time       `json:"created_at"`
	Description        string          `json:"description"`
	PaymentGateway     string          `json:"payment_gateway"`
	GatewayCommission  float64         `json:"gateway_commission"`
	PlatformCommission float64         `json:"platform_commission"`
	StoreBalanceDelta  float64         `json:"store_balance_delta"`
}