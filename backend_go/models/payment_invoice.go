package models

import (
	"time"

	"gorm.io/gorm"
)

type InvoiceStatus string

const (
	InvoiceStatusPending   InvoiceStatus = "pending"
	InvoiceStatusCompleted InvoiceStatus = "completed"
	InvoiceStatusFailed    InvoiceStatus = "failed"
)

// PaymentInvoice represents a record of a payment attempt.
type PaymentInvoice struct {
	ID        uint `gorm:"primaryKey"`
	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt gorm.DeletedAt `gorm:"index"`

	BotUserID uint    `gorm:"index"`
	BotUser   BotUser `gorm:"foreignKey:BotUserID"`

	Amount              float64
	Status              InvoiceStatus `gorm:"index"`
	Gateway             string        `gorm:"index"`
	GatewayInvoiceID    string        `gorm:"index"`
	OrderID             string        `gorm:"uniqueIndex"` // Our internal unique ID for the transaction
	BotMessageID        *int64
	WasNotificationSent bool `gorm:"default:false"`
}
