package models

import (
	"encoding/json"
	"time"

	"gorm.io/gorm"
)

type InvoiceStatus string

const (
	InvoiceStatusPending         InvoiceStatus = "pending"
	InvoiceStatusCompleted       InvoiceStatus = "completed"
	InvoiceStatusFailed          InvoiceStatus = "failed"
	InvoiceStatusAwaitingCheck   InvoiceStatus = "awaiting_check"
	InvoiceStatusManuallyConfirmed InvoiceStatus = "manually_confirmed"
)

// PaymentInvoice represents a record of a payment attempt.
type PaymentInvoice struct {
	ID        uint `gorm:"primaryKey"`
	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt gorm.DeletedAt `gorm:"index" swaggertype:"string" example:"2024-01-01T00:00:00Z"`

	BotUserID uint    `gorm:"index"`
	BotUser   BotUser `gorm:"foreignKey:BotUserID"`

	OriginalAmount      float64
	Amount              float64
	Status              InvoiceStatus   `gorm:"index"`
	Gateway             string          `gorm:"index"`
	GatewayInvoiceID    string          `gorm:"index"`
	OrderID             string          `gorm:"uniqueIndex"` // Our internal unique ID for the transaction
	PayURL              *string         `json:"pay_url"`
	PaymentDetails      json.RawMessage `gorm:"type:jsonb" json:"payment_details" swaggertype:"object"` // Store raw JSON from gateway
	BotMessageID        *int64
	WasNotificationSent bool `gorm:"default:false"`
}
